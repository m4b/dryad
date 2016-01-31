// TODO: implement the gnu symbol lookup with bloom filter
// start linking some symbols!
use std::collections::HashMap;
use std::boxed::Box;
use std::fmt;
use std::fs::File;
//use std::fs::OpenOptions;
use std::path::Path;

//use utils::*;
use binary::elf::header;
use binary::elf::program_header;
use binary::elf::dyn;
//use binary::elf::sym;
use binary::elf::rela;
use binary::elf::loader;
use binary::elf::image::{ Executable, SharedObject} ;

use kernel_block;
use auxv;
use relocate;

/// The internal config the dynamic linker generates from the environment variables it receives.
struct Config<'a> {
    bind_now: bool,
    debug: bool,
    secure: bool,
    verbose: bool,
    trace_loaded_objects: bool,
    library_path: &'a [&'a str],
    preload: &'a[&'a str]
}

impl<'a> Config<'a> {
    pub fn new<'b> (block: &'b kernel_block::KernelBlock) -> Config<'b> {
        // Must be non-null or not in environment to be "false".  See ELF spec, page 42:
        // http://flint.cs.yale.edu/cs422/doc/ELF_Format.pdf
        let bind_now = if let Some (var) = block.getenv("LD_BIND_NOW") {
            var != "" } else { false };
        let debug = if let Some (var) = block.getenv("LD_DEBUG") {
            var != "" } else { false };
        // TODO: check this, I don't believe this is valid
        let secure = block.getauxval(auxv::AT_SECURE).is_some();
        // TODO: add different levels of verbosity
        let verbose = if let Some (var) = block.getenv("LD_VERBOSE") {
            var != "" } else { false };
        let trace_loaded_objects = if let Some (var) = block.getenv("LD_TRACE_LOADED_OBJECTS") {
            var != "" } else { false };
        Config {
            bind_now: bind_now,
            debug: debug,
            secure: secure,
            verbose: verbose,
            trace_loaded_objects: trace_loaded_objects,
            //TODO: finish path logics
            library_path: &[],
            preload: &[],
        }
    }
}

impl<'a> fmt::Debug for Config<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bind_now: {} debug: {} secure: {} verbose: {} trace_loaded_objects: {} library_path: {:#?} preload: {:#?}",
               self.bind_now,
               self.debug,
               self.secure,
               self.verbose,
               self.trace_loaded_objects,
               self.library_path,
               self.preload
               )
    }
}

/// The dynamic linker
/// TODO: add lib vector or lib working_set and lib finished_set
/// TODO: Change permissions on most of these fields
pub struct Linker<'a> {
    pub base: u64,
    pub load_bias: u64,
    pub vdso: u64,
    pub ehdr: &'a header::Header,
    pub phdrs: &'a [program_header::ProgramHeader],
    pub dynamic: &'a [dyn::Dyn],
    config: Config<'a>,
    // why is this a hashmap to isize - remove
    working_set: Box<HashMap<String, SharedObject<'a>>>,
}

impl<'a> fmt::Debug for Linker<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "base: {:x} load_bias: {:x} vdso: {:x} ehdr: {:#?} phdrs: {:#?} dynamic: {:#?} Config: {:#?}",
               self.base,
               self.load_bias,
               self.vdso,
               self.ehdr,
               self.phdrs,
               self.dynamic,
               self.config
               )
    }
}

#[inline]
fn compute_load_bias(base:u64, phdrs:&[program_header::ProgramHeader]) -> u64 {
    for phdr in phdrs {
        if phdr.p_type == program_header::PT_LOAD {
            return base + (phdr.p_offset - phdr.p_vaddr);
        }
    }
    0
}

/// TODO: i think this is false; we may need to relocate R_X86_64_GLOB_DAT and R_X86_64_64
/// private linker relocation function; assumes dryad _only_
/// contains X86_64_RELATIVE relocations, which should be true
fn relocate_linker(bias: u64, relas: &[rela::Rela]) {
    for rela in relas {
        if rela::r_type(rela.r_info) == rela::R_X86_64_RELATIVE {
            // get the actual symbols address given by adding the on-disk binary offset `r_offset` to the symbol with the actual address the linker was loaded at
            let reloc = (rela.r_offset + bias) as *mut u64;
            // now set the content of this address to whatever is at the load bias + the addend
            // typically, this is all static, constant read only global data, like strings, constant ints, etc.
            unsafe {
                // TODO: verify casting bias to an i64 is correct
                *reloc = (rela.r_addend + bias as i64) as u64;
            }
        }
    }
}

extern {
    /// TLS init function with needs a pointer to aux vector indexed by AT_<TYPE> that musl likes
    fn __init_tls(aux: *const u64);
}

/// TODO:
/// 1. add config logic path based on env variables
/// 2. be able to link against linux vdso
impl<'a> Linker<'a> {
    pub fn new<'b> (base: u64, block: &'b kernel_block::KernelBlock) -> Result<Linker<'b>, &'static str> {
        unsafe {
            let ehdr = header::unsafe_as_header(base as *const u64);
            let addr = (base + ehdr.e_phoff) as *const program_header::ProgramHeader;
            let phdrs = program_header::to_phdr_array(addr, ehdr.e_phnum as usize);
            let load_bias = compute_load_bias(base, &phdrs);
            let vdso = block.getauxval(auxv::AT_SYSINFO_EHDR).unwrap();

            if let Some(dynamic) = dyn::get_dynamic_array(load_bias, &phdrs) {
                let relocations = relocate::get_relocations(load_bias, &dynamic);
                relocate_linker(load_bias, &relocations);
                // dryad has successfully relocated itself; time to init tls
                __init_tls(block.get_aux().as_ptr()); // this might not be safe yet because vec allocates

                // we relocated ourselves so it should be safe to heap allocate
                let working_set = Box::new(HashMap::new());

                Ok (Linker {
                    base: base,
                    load_bias: load_bias,
                    vdso: vdso,
                    ehdr: &ehdr,
                    phdrs: &phdrs,
                    dynamic: &dynamic,
                    config: Config::new(&block),
                    working_set: working_set,
                })
            } else {
                Err ("<dryad> SEVERE: no dynamic array found for dryad; exiting\n")
            }
        }
    }

    /// 1. Open fd to shared object ✓ - TODO: parse and use /etc/ldconfig.cache
    /// 2. get program headers ✓
    /// 3. mmap PT_LOAD phdrs ✓
    /// 4. compute load bias and base ✓
    /// 5. get _DYNAMIC real address from the mmap'd segments ✓
    /// 6a. create SharedObject from above ✓
    /// 6b. relocate the SharedObject, including GLOB_DAT
    /// 6c. resolve function and PLT; for now, just act like LD_PRELOAD is set
    /// 7. add `soname` => `SharedObject` entry in `linker.loaded`
    fn load(&mut self, soname: &str) -> Result<(), String> {
        // TODO: properly open the file using soname -> path with something like `resolve_soname`
        // TODO: if soname ∉ linker.loaded { then do this }
        // TODO: add write true?
        // match OpenOptions::new().read(true).write(false).open(Path::new("/usr/lib/").join(soname)) {
        match File::open(Path::new("/usr/lib/").join(soname)) {
            Ok (mut fd) => {

                println!("Opened: {:?}", fd);
                let shared_object = try!(loader::load(soname, &mut fd));
                self.working_set.insert(soname.to_string(), shared_object);
            },

            Err (e) => return Err(format!("<dryad> could not open {}: err {:?}", &soname, e))
        }

        Ok (())
    }

    /// Main staging point for linking the executable dryad received
    pub fn link_executable(&mut self, image: Executable) -> Result<(), String> {

        // TODO: transfer ownership of libs to the linker, so it can be parallelized
        for lib in image.needed {
            // shared_object <- load(lib);
            // if has unloaded lib deps, link(shared_object)
            try!(self.load(lib));
        }

        println!("{:#?}", self.working_set);

        // relocate the executable
        unsafe {
            // TODO: move this into image init as well
            let relas = relocate::get_relocations(image.load_bias, image.dynamic);
            println!("Relas:\n  {:#?}", relas);
            relocate::relocate(image.load_bias, relas, image.symtab, image.strtab);
        }

        Ok (())
    }
}
