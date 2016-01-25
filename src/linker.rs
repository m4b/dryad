use std::collections::HashMap;
use std::boxed::Box;
use std::fmt;
//use std::mem;
//use std::io;
//use std::prelude::*;

use std::io::Read;
use std::fs::File;

use binary::elf::header;
use binary::elf::program_header;
use binary::elf::dyn;
use binary::elf::sym;
use binary::elf::rela;
use binary::elf::loader;
use binary::elf::image::{ Executable, SharedObject} ;
use std::os::unix::io::AsRawFd;

use utils::*;
use kernel_block;
use auxv;


use relocate;

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
        let bind_now = if let Some (var) = block.getenv("LD_BIND_NOW") {
            var != "0" && var != "false" } else { false };
        let debug = if let Some (var) = block.getenv("LD_DEBUG") {
            var != "0" && var != "false" } else { false };
        // TODO: check this, I don't believe this is valid
        let secure = block.getauxval(auxv::AT_SECURE).is_some();
        let verbose = if let Some (var) = block.getenv("LD_VERBOSE") {
            var != "0" && var != "false" } else { false };
        let trace_loaded_objects = if let Some (var) = block.getenv("LD_TRACE_LOADED_OBJECTS") {
            var != "0" && var != "false" } else { false };
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

// TODO: add lib vector or lib working_set and lib finished_set
pub struct Linker<'a> {
    pub base: u64,
    pub load_bias: u64,
    pub vdso: u64,
    pub ehdr: &'a header::Header,
    pub phdrs: &'a [program_header::ProgramHeader],
    pub dynamic: &'a [dyn::Dyn],
    config: Config<'a>,
    // why is this a hashmap to isize - remove
    working_set: Box<HashMap<String, isize>>,
}

// TODO: add needed vector
// TODO: add symtab &'a [Sym] instead of u64 ?
struct LinkInfo<'a> {
    pub rela:u64,
    pub relasz:u64,
    pub relaent:u64,
    pub relacount:u64,
    pub gnu_hash:u64,
    pub hash:u64,
    pub strtab:u64,
    pub strsz:u64,
    pub symtab:u64,
    pub syment:u64,
    pub pltgot:u64,
    pub pltrelsz:u64,
    pub pltrel:u64,
    pub jmprel:u64,
    pub verneed:u64,
    pub verneednum:u64,
    pub versym:u64,
    pub init:u64,
    pub fini:u64,
    pub needed_count:usize,
    pub libs: Vec<&'a str>
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

#[inline]
fn prelink<'a> (bias: u64, dynamic: &'a [dyn::Dyn]) -> LinkInfo<'a> {
    let mut rela = 0;
    let mut relasz = 0;
    let mut relaent = 0;
    let mut relacount = 0;
    let mut gnu_hash = 0;
    let mut hash = 0;
    let mut strtab = 0;
    let mut strsz = 0;
    let mut symtab = 0;
    let mut syment = 0;
    let mut pltgot = 0;
    let mut pltrelsz = 0;
    let mut pltrel = 0;
    let mut jmprel = 0;
    let mut verneed = 0;
    let mut verneednum = 0;
    let mut versym = 0;
    let mut init = 0;
    let mut fini = 0;
    let mut needed_count = 0;
    let mut needed = vec![0; 20];
    for dyn in dynamic {
        match dyn.d_tag {
            dyn::DT_RELA => rela = dyn.d_val + bias, // .rela.dyn
            dyn::DT_RELASZ => relasz = dyn.d_val,
            dyn::DT_RELAENT => relaent = dyn.d_val,
            dyn::DT_RELACOUNT => relacount = dyn.d_val,
            dyn::DT_GNU_HASH => gnu_hash = dyn.d_val + bias,
            dyn::DT_HASH => hash = dyn.d_val + bias,
            dyn::DT_STRTAB => strtab = dyn.d_val + bias,
            dyn::DT_STRSZ => strsz = dyn.d_val,
            dyn::DT_SYMTAB => symtab = dyn.d_val + bias,
            dyn::DT_SYMENT => syment = dyn.d_val,
            dyn::DT_PLTGOT => pltgot = dyn.d_val + bias,
            dyn::DT_PLTRELSZ => pltrelsz = dyn.d_val,
            dyn::DT_PLTREL => pltrel = dyn.d_val,
            dyn::DT_JMPREL => jmprel = dyn.d_val + bias, // .rela.plt
            dyn::DT_VERNEED => verneed = dyn.d_val + bias,
            dyn::DT_VERNEEDNUM => verneednum = dyn.d_val,
            dyn::DT_VERSYM => versym = dyn.d_val + bias,
            dyn::DT_INIT => init = dyn.d_val + bias,
            dyn::DT_FINI => fini = dyn.d_val + bias,
            dyn::DT_NEEDED => {
                // this is a _dubiously_ premature optimization for likely a list of 3-4 libs
                let len = needed.len();
                if needed_count >= len {
                    needed.resize(len * 2, 0);
                }
                needed[needed_count] = dyn.d_val;
                needed_count += 1;
            },
            _ => ()
        }
    }

    let mut libs:Vec<&str> = Vec::with_capacity(needed_count as usize);
    for i in 0..needed_count as usize {
        libs.push(str_at(strtab as *const u8, needed[i] as isize));
    }

    LinkInfo {
        rela: rela,
        relasz: relasz,
        relaent: relaent,
        relacount: relacount,
        gnu_hash: gnu_hash,
        hash: hash,
        strtab: strtab,
        strsz: strsz,
        symtab: symtab,
        syment: syment,
        pltgot: pltgot,
        pltrelsz: pltrelsz,
        pltrel: pltrel,
        jmprel: jmprel,
        verneed: verneed,
        verneednum: verneednum,
        versym: versym,
        init: init,
        fini: fini,
        needed_count: needed_count,
        libs: libs
    }
}

impl<'a> fmt::Debug for LinkInfo<'a> {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rela: 0x{:x} relasz: {} relaent: {} relacount: {} gnu_hash: 0x{:x} hash: 0x{:x} strtab: 0x{:x} strsz: {} symtab: 0x{:x} syment: {} pltgot: 0x{:x} pltrelsz: {} pltrel: {} jmprel: 0x{:x} verneed: 0x{:x} verneednum: {} versym: 0x{:x} init: 0x{:x} fini: 0x{:x} libs: {:#?}",
               self.rela,
               self.relasz,
               self.relaent,
               self.relacount,
               self.gnu_hash,
               self.hash,
               self.strtab,
               self.strsz,
               self.symtab,
               self.syment,
               self.pltgot,
               self.pltrelsz,
               self.pltrel,
               self.jmprel,
               self.verneed,
               self.verneednum,
               self.versym,
               self.init,
               self.fini,
               self.libs
               )
    }
}

/// private linker relocation function; assumes dryad _only_
/// contains X86_64_RELATIVE relocations, which should be true
fn relocate_linker(bias: u64, relas: &[rela::Rela]) {
    for rela in relas {
        if rela::r_type(rela.r_info) == rela::R_X86_64_RELATIVE {
            let reloc = (rela.r_offset + bias) as *mut u64;
            // set the relocations address to the load bias + the addend
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

// TODO:
// 1. add config logic path based on env variables
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

                Ok(Linker {
                    base: base,
                    load_bias: load_bias,
                    vdso: vdso,
                    ehdr: &ehdr,
                    phdrs: &phdrs,
                    dynamic: &dynamic,
                    config: Config::new(&block),
                    working_set: Box::new(HashMap::new()) // we relocated ourselves so it should be safe to heap allocate
                })
            } else {
                Err("<dryad> SEVERE: no dynamic array found for dryad; exiting\n")
            }
        }
    }

    /// 1. Open fd to shared object ✓ - TODO: parse and use /etc/ldconfig.cache
    /// 2. get program headers ✓
    /// 3. mmap PT_LOAD phdrs ✓
    /// 4. compute load bias and base ✓
    /// 5. get _DYNAMIC real address from the mmap'd segments
    /// 6a. create SharedObject from above, by relocating
    /// 6b. resolve function and PLT; for now, just act like LD_PRELOAD is set
    /// 7. add `soname` => `SharedObject` entry in `linker.loaded`
    fn load(&self, soname: &str) -> Result<(), String> {
        // TODO: properly open the file using soname -> path with something like `resolve_soname`
        // TODO: if soname ∉ linker.loaded { then do this }
        match File::open("/usr/lib/libc.so.6") {
            Ok(mut fd) => {
                println!("Opened: {:?}", fd);
                let mut elf_header = [0; header::EHDR_SIZE];
                let _ = fd.read(&mut elf_header);

                let elf_header = header::from_bytes(&elf_header);
                let mut phdrs: Vec<u8> = vec![0; (elf_header.e_phnum as u64 * program_header::PHDR_SIZE) as usize];
                let _ = fd.read(phdrs.as_mut_slice());
                let phdrs = program_header::from_bytes(&phdrs, elf_header.e_phnum as usize);
                println!("header:\n  {:#?}\nphdrs:\n  {:#?}", &elf_header, &phdrs);
                try!(loader::load(soname, fd.as_raw_fd(), phdrs));
                /*
                unsafe {
                    if let Some(dynamic) = dyn::get_dynamic_array(0, &phdrs) {
                        println!("LOAD: header:\n  {:#?}\nphdrs:\n  {:#?}\ndynamic:\n  {:#?}", &elf_header, &phdrs, &dynamic);
                    } else {
                        return Err(format!("<dryad> no dynamic array found for {}", &soname))
                    }
                }
                 */
            },
            Err(e) => return Err(format!("<dryad> could not open {}: err {:?}", &soname, e))
        }
        Ok(())
    }

    /// Main staging point for linking the main executable
    /// 1. Get dynamic, symtab, and strtab
    /// 2. Construct initial lib set from the DT_NEEDED, and DT_NEEDED_SIZE, and go from there
    pub fn link_executable(&self, image: Executable) -> Result<(), String> {
        if let Some(dynamic) = image.dynamic {
            let link_info = prelink(image.load_bias, dynamic);
            println!("LinkInfo:\n  {:#?}", &link_info);
            let num_syms = ((link_info.strtab - link_info.symtab) / link_info.syment) as usize; // this _CAN'T_ generally be valid; but rdr has been doing it and scans every linux shared object binary without issue... so it must be right!
            let symtab = sym::get_symtab(link_info.symtab as *const sym::Sym, num_syms);
            let strtab = link_info.strtab as *const u8;
            println!("Symtab:\n  {:#?}", &symtab);

            for lib in link_info.libs {
                // shared_object <- load(lib);
                // if has unloaded lib deps, link(shared_object)
                try!(self.load(lib));
            }

            // relocate the executable
            unsafe {
                let relas = relocate::get_relocations(image.load_bias, dynamic);
                println!("Relas:\n  {:#?}", relas);
                relocate::relocate(image.load_bias, relas, symtab, strtab);
            }

            Ok(())

        } else {
            Err(format!("<dryad> Error: {} contains no _DYNAMIC", image.name))
        }
    }
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
