#![allow(unused_assignments)] // remove this after validating reloc for get_linker_relocations
// Questions from README:
// 1. Is the `reloc` _always_ in a `PT_LOAD` segment?
// 2. Is the `strtab` _always_ after the `symtab` in terms of binary offset, and hence we can compute the size of the symtab by subtracting the two?
// TODO:
// 1. fix TLS
// 2. determine reason for libc crashes again :/
use std::collections::HashMap;
use std::boxed::Box;
use std::fmt;
use std::mem;
use std::fs::File;
use std::path::Path;

extern crate crossbeam;

use elf::header::Header;
use elf::program_header::{self, ProgramHeader};
use elf::dyn;
use elf::reloc;
use elf::sym;
use loader;
use image::{self, SharedObject};
use elf::gnu_hash;

use gdb;
use utils;
use kernel_block;
use auxv;
use runtime;
use tls;
use relocation;

//thread_local!(static FOO: u32 = 0xdeadbeef);

/// The internal config the dynamic linker generates from the environment variables it receives.
struct Config<'a> {
    show_auxv: bool,
    bind_now: bool,
    debug: bool,
    secure: bool,
    verbose: bool,
    trace_loaded_objects: bool,
    library_path: Vec<&'a str>,
    preload: &'a[&'a str]
}

impl<'a> Config<'a> {
    pub fn new<'b> (block: &'b kernel_block::KernelBlock) -> Config<'b> {
        // Must be non-null or not in environment to be "false".  See ELF spec, page 42:
        // http://flint.cs.yale.edu/cs422/doc/ELF_Format.pdf
        let show_auxv = if let Some (var) = block.getenv("LD_SHOW_AUXV") {
            var != "" } else { false };
        let bind_now = if let Some (var) = block.getenv("LD_BIND_NOW") {
            var != "" } else { false };
        let debug = if let Some (var) = block.getenv("LD_DEBUG") {
            var != "" } else { false };
        // because travis is a PoS
        let debug = if let Some (var) = block.getenv("LD_DRYAD_DEBUG") {
            var == "1" } else { true };
         // TODO: FIX THIS IS NOT VALID and massively unsafe
        let secure = block.getauxval(auxv::AT_SECURE).unwrap() != 0;
        // TODO: add different levels of verbosity
        let verbose = if let Some (var) = block.getenv("LD_VERBOSE") {
            var != "" } else { false };
        let trace_loaded_objects = if let Some (var) = block.getenv("LD_TRACE_LOADED_OBJECTS") {
            var != "" } else { false };
        let library_path =
            if let Some (paths) = block.getenv("LD_LIBRARY_PATH") {
                // we don't need to allocate since technically the strings are preallocated in the environment variable, but being lazy for now
                let mut dirs: Vec<&str> = vec![];
                if !secure {
                    dirs.extend(paths.split(":").collect::<Vec<&str>>());
                }
                dirs.push("/usr/lib");
                dirs
            } else { 
                vec!["/usr/lib"]
            };
        Config {
            show_auxv: show_auxv,
            bind_now: bind_now,
            debug: debug,
            secure: secure,
            verbose: verbose,
            trace_loaded_objects: trace_loaded_objects,
            //TODO: finish path logics
            library_path: library_path,
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

/*
#[no_mangle]
pub extern fn _dryad_fini() {
    return
}
*/

/// The dynamic linker
/// TODO: Change permissions on most of these fields
pub struct Linker<'process> {
    // TODO: maybe remove base
    pub base: usize,
    pub load_bias: usize,
    pub ehdr: &'process Header,
    pub phdrs: &'process [program_header::ProgramHeader],
    pub dynamic: &'process [dyn::Dyn],
    auxv: Vec<usize>,
    config: Config<'process>,
    working_set: Box<HashMap<String, SharedObject<'process>>>, // TODO: we can eventually drop this or have it stack local var instead of field
    link_map_order: Vec<String>,
    link_map: Vec<SharedObject<'process>>,
    gdb: &'process mut gdb::Debug,
    lachesis: tls::Lachesis, // our tls delegate
}

impl<'process> fmt::Debug for Linker<'process> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "base: {:x} load_bias: {:x} ehdr: {:#?} phdrs: {:#?} dynamic: {:#?} Config: {:#?}",
               self.base,
               self.load_bias,
               self.ehdr,
               self.phdrs,
               self.dynamic,
               self.config
               )
    }
}

impl<'process> Linker<'process> {
    pub fn new<'kernel> (base: usize, block: &'kernel kernel_block::KernelBlock) -> Result<Linker<'kernel>, &'static str> {
        unsafe {

            let ehdr = &*(base as *const Header);
            let addr = (base + ehdr.e_phoff as usize) as *const program_header::ProgramHeader;
            let phdrs = ProgramHeader::from_raw_parts(addr, ehdr.e_phnum as usize);
            let load_bias = image::compute_load_bias_wrapping(base, &phdrs);
            if let Some(dynamic) = dyn::from_phdrs(load_bias, &phdrs) {
                let info = dyn::DynamicInfo::new(&dynamic, load_bias);
                relocation::relocate_linker(load_bias, &info, &phdrs);
                // dryad has successfully relocated itself; time to init tls
                let mut auxv = auxv::from_raw(block.auxv);
                auxv[auxv::AT_PHDR] = addr as usize;
                // again, one day we'll init lachesis and tls for the duration of dryad relocation+linking, using our custom hand-rolled TLS impl
                // tls::Lachesis::init_from_phdrs(load_bias as usize, phdrs);
                tls::__init_tls(auxv.as_ptr()); // this _should_ be safe since vec only allocates and shouldn't access tls. maybe.

                // we relocated ourselves so it should be safe to init the gdb debug protocols, use global data, reference static strings, call sweet functions, etc.
                utils::set_panic(); // set this as early as we can

                // init and setup gdb
                let gdb = &mut gdb::_r_debug;
                gdb.relocated_init(base);

                let config = Config::new(&block);
                let debug = config.debug;
                dbg!(debug, "init dryad with load_bias: 0x{:x}", load_bias);
                let mut working_set = Box::new(HashMap::new());
                let mut link_map_order = Vec::new();
                let link_map = Vec::new();

                //let soname = utils::str_at(soname, 0);

                if let Some(vdso_addr) = block.getauxval(auxv::AT_SYSINFO_EHDR) {
                    let vdso = SharedObject::from_raw(vdso_addr);
                    dbg!(config.debug, "loaded vdso {} at 0x{:x}", vdso.name(), vdso.load_bias);
                    link_map_order.push(vdso.name().to_string());
                    working_set.insert(vdso.name().to_string(), vdso);
                };

                Ok (Linker {
                    base: base,
                    load_bias: load_bias,
                    ehdr: &ehdr,
                    phdrs: &phdrs,
                    dynamic: &dynamic,
                    config: config,
                    working_set: working_set,
                    link_map_order: link_map_order,
                    link_map: link_map,
                    auxv: auxv,
                    gdb: gdb,
                    lachesis: tls::Lachesis::new(debug),
                })

            } else {

                Err ("Error: no dynamic array found for dryad. Why? This should be impossible, and it means someone has tampered with my existence... will try to exit, unless they tampered with that too\n")
            }
        }
    }

    /// Maybe returns the symbol which matches the name, and the SharedObject in which was found
    fn find_symbol(&self, name: &str) -> Option<(&sym::Sym, &SharedObject)> {
        // actually, this is an unfair optimization; the library might use a different hash system, like sysv
        // in which case we can't pre-hash using gnu_hash, unless we assume every lib uses gnu_hash :/
        let hash = gnu_hash::hash(name);
        for so in &self.link_map {
            if let Some(sym) = so.find(name, hash) {
                return Some ((sym, &so))
            }
        }
        None
    }

    /// Following the steps below, the dynamic linker and the program "cooperate"
    /// to resolve symbolic references through the procedure linkage table and the global
    /// offset table.
    ///
    /// 1. When first creating the memory image of the program, the dynamic linker
    /// sets the second and the third entries in the global offset table to special
    /// values. Steps below explain more about these values.
    ///
    /// 2. Each shared object file in the process image has its own procedure linkage
    /// table, and control transfers to a procedure linkage table entry only from
    /// within the same object file.
    ///
    /// 3. For illustration, assume the program calls `name1`, which transfers control
    /// to the label `.PLT1`.
    ///
    /// 4. The first instruction jumps to the address in the global offset table entry for
    /// `name1`. Initially the global offset table holds the address of the following
    /// pushq instruction, not the real address of `name1`.
    ///
    /// 5. Now the program pushes a relocation index (index) on the stack. The relocation
    /// index is a 32-bit, non-negative index into the relocation table addressed
    /// by the `DT_JMPREL` dynamic section entry. The designated relocation entry
    /// will have type `R_X86_64_JUMP_SLOT`, and its offset will specify the
    /// global offset table entry used in the previous jmp instruction. The relocation
    /// entry contains a symbol table index that will reference the appropriate
    /// symbol, `name1` in the example.
    ///
    /// 6. After pushing the relocation index, the program then jumps to `.PLT0`, the
    /// first entry in the procedure linkage table. The pushq instruction places the
    /// value of the second global offset table entry (GOT+8) on the stack, thus giving
    /// the dynamic linker one word of identifying information. The program
    /// then jumps to the address in the third global offset table entry (GOT+16),
    /// which transfers control to the dynamic linker.
    ///
    /// 7. When the dynamic linker receives control, it unwinds the stack, looks at
    /// the designated relocation entry, finds the symbol’s value, stores the "real"
    /// address for `name1` in its global offset table entry, and transfers control to
    /// the desired destination.
    ///
    /// 8. Subsequent executions of the procedure linkage table entry will transfer
    /// directly to `name1`, without calling the dynamic linker a second time. That
    /// is, the jmp instruction at `.PLT1` will transfer to `name1`, instead of "falling
    /// through" to the pushq instruction.
    fn prepare_got<'a> (&self, idx: usize, pltgot: *const usize, name: &'a str) {

        if pltgot.is_null() {
            dbg!(self.config.debug, "empty pltgot for {}", name);
            return
        }

        let len = self.link_map.len();
        let rndzv = Box::new(runtime::Rendezvous { idx: idx, debug: self.config.debug, link_map: self.link_map.as_slice() });

        unsafe {
            // got[0] == the program's address of the _DYNAMIC array, equal to address of the PT_DYNAMIC.ph_vaddr + load_bias
            // got[1] == "is the pointer to a data structure that the dynamic linker manages. This data structure is a linked list of nodes corresponding to the symbol tables for each shared library linked with the program. When a symbol is to be resolved by the linker, this list is traversed to find the appropriate symbol."
            let second_entry = pltgot.offset(1) as *mut *mut runtime::Rendezvous;
            // got[2] == the dynamic linker's runtime symbol resolver
            let third_entry = pltgot.offset(2) as *mut usize;

            *second_entry = Box::into_raw(rndzv);
            *third_entry = runtime::_dryad_resolve_symbol as usize;
            dbg!(self.config.debug, "finished got setup for {} GOT[1] = {:?} GOT[2] = {:#x}", name, *second_entry, *third_entry);
        }

    }

    #[inline(always)]
    fn find_provider(&self, sym: &sym::Sym, name: &str) -> Option<&SharedObject> {
        if sym.st_info != sym::STB_LOCAL {
            let hash = gnu_hash::hash(name);
            for ref so in &self.link_map {
                if let Some(sym) = so.find(name, hash) {
                    return Some(so);
                }
            }
        }
        None
    }

    // TODO: reloc::R_X86_64_GLOB_DAT => this is a symbol resolution and requires full link map data, and _cannot_ be done before everything is relocated
    // ditto TPOFF64...
    fn relocate_got (&self, idx: usize, so: &SharedObject) {
        let symtab = &so.symtab;
        let strtab = &so.strtab;
        let bias = so.load_bias;
        let mut count = 0;
        let tls = so.tls;
        if so.link_info.textrel {
            let res = utils::mmap::mprotect_phdrs(&so.phdrs, bias, utils::mmap::PROT_WRITE);
        }
        for reloc in so.relocations {
            let typ = reloc::r_type(reloc.r_info);
            let sym = reloc::r_sym(reloc.r_info); // index into the sym table
            let symbol = &symtab[sym as usize];
            let name = &strtab[symbol.st_name as usize];
            let addr = (reloc.r_offset as usize + bias) as *mut usize;
            //dbg!(self.config.debug, "reloc {:p} -> {:x}", addr, unsafe { *addr });
            match typ {
                // B + A
                relocation::RELATIVE => {
                    #[cfg(target_pointer_width = "64")]
                    let addend = reloc.r_addend as isize;
                    #[cfg(target_pointer_width = "32")]
                    let addend = unsafe { (*addr) } as isize ;

                    // set the relocations address to the load bias + the addend
                    unsafe { *addr = (addend + bias as isize) as usize; }
                    //dbg!(self.config.debug, "after reloc {:p} -> {:x}", addr, unsafe { *addr });
                    count += 1;
                },
                // S
                relocation::GLOB_DAT => {
                    // resolve symbol;
                    // 1. start with exe, then next in needed, then next until symbol found
                    // 2. use gnu_hash with symbol name to get sym info
                    if let Some((symbol, so)) = self.find_symbol(name) {
                        // TODO: add 32-bit relocation
                        #[cfg(target_pointer_width = "64")]
                        unsafe { *addr = symbol.st_value as usize + so.load_bias; }
                        count += 1;
                    }
                },
                // ========= Platform specific relocations go here =========
                #[cfg(arch = "x86_64")]
                // (S + A) - offset
                reloc::R_X86_64_TPOFF64 => {
                    if let Some((symbol, providing_so)) = self.find_symbol(name) {
                        let tls = providing_so.tls.expect(&format!("Error: symbol \"{}\" required in {}, but the providing so {} does not have a TLS program header", name, so.name(), providing_so.name()));
                        // TODO: it should be the symbol value (= tls offset in that module) plus the addend + the tls offset into the dtv of that module; i don't think load bias is used at all here, as it will be a relative got load?
                        unsafe { *addr = (symbol.st_value as i64 + reloc.r_addend as i64 - tls.offset as i64) as usize; }
                        dbgc!(purple_bold: self.config.debug, "tls", "bound {} \"{}\" required in {} to provider {} with address 0x{:x}", sym::get_type(symbol.st_info), name, so.name(), providing_so.name(), unsafe { *addr });
                        count += 1;
                    }
                },
                #[cfg(arch = "x86_64")]
                // S + A
                reloc::R_X86_64_64 => {
                    // TODO: this is inaccurate because find_symbol is inaccurate
                    if let Some((symbol, so)) = self.find_symbol(name) {
                        unsafe { *addr = (reloc.r_addend + symbol.st_value as i64 + so.load_bias as i64) as usize; }
                        count += 1;
                    }
                },
                // TODO: add erro checking
                _ => ()
            }
        }

        dbg!(self.config.debug, "relocated {} symbols in {}", count, &so.name());

        self.prepare_got(idx, so.pltgot, &so.name());
    }

    /// TODO: add check for if SO has the DT_BIND_NOW, and also other flags...
    fn relocate_plt (&self, so: &SharedObject) {

        let symtab = &so.symtab;
        let strtab = &so.strtab;
        let bias = so.load_bias;
        let mut count = 0;

        // x86-64 ABI, pg. 78:
        // > Much as the global offset table redirects position-independent address calculations
        // > to absolute locations, the procedure linkage table redirects position-independent
        // > function calls to absolute locations.
        for reloc in so.pltrelocations {
            let typ = reloc::r_type(reloc.r_info);
            let sym = reloc::r_sym(reloc.r_info); // index into the sym table
            let symbol = &symtab[sym as usize];
            let name = &strtab[symbol.st_name as usize];
            let addr = (reloc.r_offset as usize + bias) as *mut usize;
            //dbg!(self.config.debug, "reloc {:p} -> {:x}", addr, unsafe { *addr });
            match typ {
                relocation::JUMP_SLOT if self.config.bind_now => {
                    if let Some((symbol, so)) = self.find_symbol(name) {
                        unsafe { *addr = symbol.st_value as usize + so.load_bias; }
                        count += 1;
                    } else {
                        dbgc!(orange_bold: self.config.debug, "dryad.warning", "no resolution for {}", name);
                    }
                },
                // fun @ (B + A)()
                relocation::IRELATIVE => {
                    #[cfg(target_pointer_width = "64")]
                    let addend = reloc.r_addend as isize;
                    #[cfg(target_pointer_width = "32")]
                    let addend = unsafe { (*addr) } as isize ;

                    let ifunc_addr = addend + bias as isize;
//                    dbg!(self.config.debug, "irelative: bias: {:#x} addend: {:#x} addr: {:#x}", bias, reloc.r_addend, addr);
                    unsafe {
                        let ifunc = mem::transmute::<usize, (fn() -> usize)>(ifunc_addr as usize);
                        *addr = ifunc() as usize;
//                        dbg!(self.config.debug, "ifunc addr: 0x{:x}", *reloc);
                    }
                    count += 1;
                },
                // TODO: add error checking
                _ => ()
            }
        }
        if so.link_info.textrel {
            let res = utils::mmap::mprotect_phdrs(&so.phdrs, bias, 0);
        }
        dbg!(self.config.debug, "relocate plt: {} symbols for {}", count, so.name());
    }

    /// TODO: rename to something like `load_all` to signify on return everything has loaded?
    /// So: load many -> join -> relocate many -> join -> relocate executable and transfer control
    /// 1. Open fd to shared object ✓ - TODO: parse and use /etc/ldconfig.cache
    /// 2. get program headers ✓
    /// 3. mmap PT_LOAD phdrs ✓
    /// 4. compute load bias and base ✓
    /// 5. get _DYNAMIC real address from the mmap'd segments ✓
    /// 6a. create SharedObject from above ✓
    /// 6b. relocate the SharedObject, including GLOB_DAT ✓ TODO: TLS shite
    /// 6c. resolve function and PLT; for now, just act like LD_PRELOAD is set
    /// 7. add `soname` => `SharedObject` entry in `linker.loaded` TODO: use better structure, resolve dependency chain
    fn load(&mut self, soname: &str) -> Result<(), String> {
        // TODO: properly open the file using soname -> path with something like `resolve_soname`
        let paths = self.config.library_path.to_owned(); // TODO: so we compile, fix unnecessary alloc

        // soname ∉ linker.loaded
        if !self.working_set.contains_key(soname) {
            let mut found = false;
            for path in paths {
                let file = Path::new(&path).join(soname);
                match File::open(&file) {
                    Ok (mut fd) => {
                        found = true;
                        dbg!(self.config.debug, "opened: {:?}", fd);
                        let shared_object = try!(loader::load(soname, file.to_string_lossy().into_owned(), &mut fd,  self.config.debug, &mut self.lachesis));
                        unsafe { self.gdb.add_so(&shared_object); }

                        let libs = &shared_object.libs.to_owned(); // TODO: fix this unnecessary allocation, but we _must_ insert before iterating
                        self.working_set.insert(soname.to_string(), shared_object);

                        // breadth first addition, and unnecessary amount of searching but who cares for now
                        // this also fixes the snappy dedup problem
                        for lib in libs {
                            let mut is_elem = false;
                            for lib2 in &self.link_map_order {
                                if lib2 == lib { is_elem = true; break }
                            }
                            if !is_elem { self.link_map_order.push(lib.to_string());}
                        }

                        for lib in libs {
                            try!(self.load(lib));
                        }
                        break
                    },
                    _ => (),
                }
            }
            if !found {
                return Err(format!("Error: could not find {} in {:?}", &soname, self.config.library_path))
            }
        }

        Ok (())
    }
    
    /// Main staging point for linking the executable dryad received
    /// (Experimental): Responsible for parallel execution and thread joining
    /// 1. First builds the executable and then all the shared object dependencies and joins the result
    /// 2. Then, creates the link map, and then relocates all the shared object dependencies and joins the result
    /// 3. Finally, relocates the executable, and then transfers control
    pub fn link(mut self, block: &kernel_block::KernelBlock) -> Result<(), String> {

//        dbg!(self.config.debug, "I am that I am:\n  {:#?}", &self);
        /*
        let array = [1, 2, 3];
        crossbeam::scope(|scope| {
            for i in &array {
                scope.spawn(move || {
                    //println!("crossbeam says: {}", i);
                });
            }
        });
        */

        // build executable
        dbgc!(red: self.config.debug, "dryad", "loading executable");
        let name = utils::str_at(block.argv[0], 0);
        let phdr_addr = block.getauxval(auxv::AT_PHDR).unwrap();
        let phnum  = block.getauxval(auxv::AT_PHNUM).unwrap();
        let image = try!(SharedObject::from_executable(name, phdr_addr, phnum, &mut self.lachesis));

        dbg!(self.config.debug, "Main Image:\n  {:#?}", &image);

        // 1. load all

        // TODO: transfer ownership of libs (or allocate) to the linker, so it can be parallelized
        // this is the only obvious candidate for parallelization, and it's dubious at best... but large binaries spend 20% of time loading and 80% on relocation
        self.link_map_order.extend(image.libs.iter().map(|s| s.to_string()));
        unsafe {
            // insert the _r_debug struct into the executables _DYNAMIC array
            // this is unsafe because we use pointers because I don't feel like changing every borrowed reference for the dynamic array to a mutable borrow for one single time for the whole program duration that the _DYNAMIC array ever gets mutated
            gdb::insert_r_debug(image.dynamic);
            self.gdb.update(gdb::State::RT_ADD);
        }
        for lib in &image.libs {
            try!(self.load(lib));
        }
        unsafe {
            // we need to read-add dryad otherwise gdb likes to unload it for some reason i have yet to determine; this is a hack.  See:
            // https://github.com/m4b/dryad/issues/4
            // TODO: remove hardcoded /tmp/dryad.so.1 and use soname instead
            gdb::LinkMap::append(Box::into_raw(Box::new(gdb::LinkMap::new(self.load_bias, "/tmp/dryad.so.1", self.dynamic))), self.gdb.r_map);
            self.gdb.update(gdb::State::RT_CONSISTENT);
        }

        dbg!(self.config.debug, "link_map_order: {:#?}", self.link_map_order);

        self.link_map.reserve_exact(self.link_map_order.len()+1);
        self.link_map.push(image);
        // TODO: we should go in reverse order like glibc ?
        for soname in &self.link_map_order {
            let so = self.working_set.remove(soname).unwrap();
            self.link_map.push(so);
        }
        dbg!(self.config.debug, "working set is drained: {}", self.working_set.len() == 0);
        dbg!(self.config.debug, "link_map ptr: {:#?}, cap = len: {}", self.link_map.as_ptr(), self.link_map.capacity() == self.link_map.len());
        // <join>
        // 2. relocate all
        // TODO: after _all_ SharedObject have been loaded, it is safe to relocate if we stick to ELF symbol search rule of first search executable, then in each of DT_NEEDED in order, then deps of first DT_NEEDED, and if not found, then deps of second DT_NEEDED, etc., i.e., breadth-first search.  Why this is allowed to continue past the executable's _OWN_ dependency list is anyone's guess; a penchant for chaos perhaps?

        // SCOPE is resolved breadth first by ld-so and flattened to a single search list (more or less)
        // exe
        // |_ libfoo
        // |_ libbar
        //    |_ libderp
        //    |_ libbaz
        //    |_ libfoo
        //    |_ libslerp
        //    |_ libmerp
        // |_ libbaz
        //    |_ libmerp
        //    |_ libslerp
        // |_
        //
        // is reduced to [exe, libfoo, libbar, libbaz, libderp, libslerp, libmerp]

        // TODO: determine ld-so's relocation order (_not_ equivalent to it's search order, which is breadth first from needed libs)
        // Because gnu_ifuncs essentially execute arbitrary code, including calling into the GOT, if the GOT isn't setup and relative relocations, for example, haven't been processed in the binary which has the reference, we're doomed.  Example is a libm ifunc (after matherr) for `__exp_finite` that calls `__get_cpu_features` which resides in libc.

        for (i, so) in self.link_map.iter().enumerate() {
            self.relocate_got(i, so);
        }

        // I believe we can parallelize the relocation pass by:
        // 1. skipping constructors, or blocking until the linkmaps deps are signalled as finished
        // 2. if skip, rerun through the link map again and call each constructor, since the GOT was prepared and now dynamic calls are ready
        for so in self.link_map.iter() {
            self.relocate_plt(so);
        }

//        println!("libc: {:#?}", unsafe { &::tls::__libc});
        // <join>
        // 3. transfer control

        // we safely loaded and relocated everything, dryad will now forget itself
        // so the structures we setup don't segfault when we try to access them back again after passing through assembly to `dryad_resolve_symbol`,
        // which from the compiler's perspective means they needs to be dropped
        // "Blessed are the forgetful, for they get the better even of their blunders."
        dbg!(self.config.debug, "\"Without forgetting it is quite impossible to live at all.\"");
        if !self.config.secure && self.config.show_auxv {
            auxv::show(&self.auxv);
        }

        type InitFn = fn (argc: isize, argv: *const *const u8, env: *const *const u8) -> ();
        for so in self.link_map.iter() {
//            dbg!(self.config.debug, "{}: init: 0x{:x} - 0x{:x} = 0x{:x}", so.name(), so.link_info.init, so.load_bias, so.link_info.init.wrapping_sub(so.load_bias));
            let (argc, argv, envp) = (block.argc, block.argv.as_ptr(), block.env.as_ptr());
            if so.link_info.init != 0 {
                let init = unsafe { mem::transmute::<usize, InitFn>(so.link_info.init as usize)};
                init(argc, argv, envp);
                let init_arr = so.link_info.init_array as *mut InitFn;
                let sz = so.link_info.init_arraysz as usize;
                let count = sz / mem::size_of::<usize>();
                for i in 0..count {
                    unsafe {
                        let init = mem::transmute::<usize, InitFn>(*init_arr.offset(i as isize) as usize);
                        init(argc, argv, envp);
                    }
                }
            }
        }

        unsafe {
            // one day we will init_tls using lachesis - but it is not this day!
            // // ::tls::init_tls(self.lachesis.current_modid, &mut self.lachesis.modules);
            // this will be the cool way to do it
            // tls::Lachesis::init_from_phdrs(self.load_bias as usize, self.phdrs);
// calling this with the program header of the entry runs it as normal
// except since libc isn't properly initialized (__libc_malloc_initialized == 0), it tries to load dynamically and crashes since none of the rtld_global struct is setup :/
            let auxv = auxv::from_raw(block.auxv);
            tls::__init_tls(auxv.as_ptr());
        }
        mem::forget(self);
        Ok (())
    }
}
