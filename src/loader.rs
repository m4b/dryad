/// TODO: parse and return flags per DSO, add as entry to the struct
/// TODO: fix the high address mapperr for `__libc_start_main`

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::slice;
use std::os::raw::{c_int};

use utils::mmap;
use utils::page;
use image::SharedObject;
use binary::elf::header;
use binary::elf::program_header;
use binary::elf::dyn;
use binary::elf::sym;
use binary::elf::rela;
use binary::elf::strtab::Strtab;
use binary::elf::gnu_hash::GnuHash;

extern {
    /// libc #defines errno *(__errno_location()) ... so errno isn't a symbol in the actual binary and accesses will segfault us. yay.
    fn __errno_location() -> *const i32;
}

#[inline(always)]
fn get_errno () -> i32 {
    unsafe { *__errno_location() }
}

// TODO: add safe adds, there's ridiculous amounts of casting everywhere
#[inline(always)]
fn map_fragment(fd: &File, base: u64, offset: u64, size: usize) -> Result<(u64, usize, *const u64), String> {
    let offset = base + offset;
    let page_min = page::page_start(offset);
    let end_offset = offset + size as u64;
    let end_offset = end_offset + page::page_offset(offset);

    let map_size: usize = (end_offset - page_min) as usize;

    if map_size < size {
        return Err (format!("<dryad> Error: file {:#?} has map_size = {} < size = {}, aborting", fd, map_size, size))
    }

    let map_start = unsafe { mmap::mmap(0 as *const u64,
                                        map_size,
                                        mmap::PROT_READ,
                                        mmap::MAP_PRIVATE as c_int,
                                        fd.as_raw_fd() as c_int,
                                        page_min as usize) };

    if map_start == mmap::MAP_FAILED {

        Err (format!("<dryad> Error: mmap failed for {:#?} with errno {}, aborting", fd, get_errno()))

    } else {

        let data = (map_start + page::page_offset(offset)) as *const u64;
        Ok ((map_start, map_size, data))
    }
}

#[inline(always)]
fn mmap_dynamic<'a> (soname: &str, fd: &File, phdrs: &[program_header::ProgramHeader]) -> Result<&'a[dyn::Dyn], String>{
    for phdr in phdrs {
        if phdr.p_type == program_header::PT_DYNAMIC {
            // mmap
            let (dynamic_start, dynamic_size, dynamic_data) = try!(map_fragment(fd, 0, phdr.p_offset, phdr.p_filesz as usize));
            // iter
            let dyn_ptr = dynamic_data as *const dyn::Dyn;
            let mut i = 0;
            unsafe {
                while (*dyn_ptr.offset(i)).d_tag != dyn::DT_NULL {
                    i += 1;
                }
                // slice and tie the lifetime to the mmap
                return Ok (slice::from_raw_parts(dyn_ptr, (i+1) as usize))
            }
        }
    }
    Err (format!("<dryad> No dynamic section for {}", soname))
}

#[inline(always)]
fn compute_load_size (phdrs: &[program_header::ProgramHeader]) -> (usize, u64, u64) {
    let mut max_vaddr = 0;
    let mut min_vaddr = 0;
    for phdr in phdrs {

        if phdr.p_type != program_header::PT_LOAD {
            continue
        }

        if phdr.p_vaddr < min_vaddr {
            min_vaddr = phdr.p_vaddr;
        }

        if phdr.p_vaddr + phdr.p_memsz > max_vaddr {
            max_vaddr = phdr.p_vaddr + phdr.p_memsz;
        }
    }

    min_vaddr = page::page_start(min_vaddr);
    max_vaddr = page::page_end(max_vaddr);

    ((max_vaddr - min_vaddr) as usize, min_vaddr, max_vaddr)
}

#[inline(always)]
fn reserve_address_space (phdrs: &[program_header::ProgramHeader]) -> Result <(u64, u64, u64), String> {

    let (size, min_vaddr, max_vaddr) = compute_load_size(&phdrs);

    let mmap_flags = mmap::MAP_PRIVATE | mmap::MAP_ANONYMOUS;
    let start = unsafe { mmap::mmap(0 as *const u64,
                                    size,
                                    // TODO: this is _UNSAFE_:
                                    // for now, using PROT_NONE seems to give SEGV_ACCERR on execution of PT_LOAD mmaped segments (i.e., operation not allowed on mapped object)
                                    mmap::PROT_EXEC | mmap::PROT_READ | mmap::PROT_WRITE,
//                                    mmap::PROT_NONE,
                                    mmap_flags as c_int,
                                    -1,
                                    0) };

    if start == mmap::MAP_FAILED {

        Err (format!("<dryad> Failure: anonymous mmap failed for size {:x} with errno {}", size, get_errno()))

    } else {

        let load_bias = start - min_vaddr;
        let end = start + size as u64;
        Ok ((start, load_bias, end))
    }
}

#[inline(always)]
fn pflags_to_prot (x: u32) -> isize {
    use binary::elf::program_header::{PF_X, PF_R, PF_W};

    // I'm a dick for writing this/copying maniac C programmer implementations: but it checks the flag to see if it's the PF value,
    // and returns the appropriate mmap version, and logical ORs this for use in the mmap prot argument
    (if x & PF_X == PF_X { mmap::PROT_EXEC } else { 0 }) |
    (if x & PF_R == PF_R { mmap::PROT_READ } else { 0 }) |
    (if x & PF_W == PF_W { mmap::PROT_WRITE } else { 0 })
}

// MAJOR TODO: I believe gdb is messed up due to
// 1. program headers not being mmap'd in expected location
// 2. dynamic array + strtab's not being in expected location w.r.t. program headers
/// Loads an ELF binary from the given fd, mmaps its contents, and returns a SharedObject, whose lifetime is tied to the mmap's, i.e., manually managed
/// TODO: probably just move this function to image and use it as the impl
pub fn load<'a> (soname: &str, load_path: String, fd: &mut File, debug: bool) -> Result <SharedObject<'a>, String> {
    // 1. Suck up the elf header and construct the program headers
    let ehdr = header::Header::from_fd(fd).map_err(|e| format!("<dryad> Error {:?}", e))?;
    let phdrs = program_header::ProgramHeader::from_fd(fd, ehdr.e_phoff, ehdr.e_phnum as usize).map_err(|e| format!("<dryad> Error {:?}", e))?;

    // 2. Reserve address space with anon mmap
    let (start, load_bias, end) = try!(reserve_address_space(&phdrs));
    if debug { println!("<loader> reserved {:#x} - {:#x} with load_bias: 0x{:x}", start, end, load_bias); }

    // TODO: swap location of this after the load bias computation so we can construct the LinkInfo with the load bias and not worry about other nonsense
    // 1.5 mmap the dynamic array with the strtab so we can access them and resolve symbol lookups against this library; this will require mmapping the segments, and storing the dynamic array, along with the strtab; TODO: benchmark against sucking them up ourselves into memory and resolve queries against that way -- probably slower...

    let dynamic = try!(mmap_dynamic(soname, &fd, &phdrs));
    let link_info = dyn::LinkInfo::new(&dynamic, 0);

    // now get the strtab from the dynamic array
    let (strtab_start, strtab_size, strtab_data) = try!(map_fragment(&fd, 0, link_info.strtab as u64, link_info.strsz));
    let strtab = unsafe { Strtab::from_raw(strtab_data as *const u8, link_info.strsz as usize) };

    let libs = dyn::get_needed(dynamic, &strtab, link_info.needed_count);

    let symtab_ptr = link_info.symtab as *const sym::Sym;
    // let (symtab_start, symtab_size, symtab_data) = try!(map_fragment(&fd, 0, link_info.symtab, 2192 * sym::SIZEOF_SYM as u64));
    let (symtab_start, symtab_size, symtab_data) = try!(map_fragment(&fd, 0, link_info.symtab as u64, (link_info.strtab - link_info.symtab) as usize));

    let num_syms = (link_info.strtab - link_info.symtab) / sym::SIZEOF_SYM;
    // TODO: probably remove this?, and add unsafe
    let symtab = unsafe { sym::from_raw(symtab_data as *const sym::Sym, num_syms) };

    // semi-hack with adding the load bias right now, but probably fine
    let relatab = unsafe { rela::from_raw((link_info.rela + load_bias as usize) as *const rela::Rela, link_info.relasz) };

    let pltrelatab = unsafe { rela::from_raw((link_info.jmprel + load_bias as usize) as *const rela::Rela, link_info.pltrelsz) };

    // TODO: place this in a separate function
    // 3. mmap the PT_LOAD program headers
    for phdr in phdrs {

        if phdr.p_type != program_header::PT_LOAD {
            continue
        }

        // TODO: add a boolean switch to know there were actually `PT_LOAD` sections, and `Err` otherwise

        let seg_start: u64 = phdr.p_vaddr + load_bias;
        let seg_end: u64   = seg_start + phdr.p_memsz;

        let seg_page_start: u64 = page::page_start(seg_start);
        let seg_page_end: u64   = page::page_start(seg_end);

        // TODO: figure this out
        let seg_file_end: u64 = seg_start + phdr.p_filesz;

        // File offsets.
        let file_start: u64 = phdr.p_offset;
        let file_end: u64   = file_start + phdr.p_filesz;

        let file_page_start = page::page_start(file_start);
        let file_length: u64 = file_end - file_page_start;

        // TODO: add error checking, if file size <= 0, if file_end greater than file_size, etc.

        if debug { println!("<loader> PT_LOAD:\n\tseg_start: {:x} seg_end: {:x} seg_page_start: {:x} seg_page_end: {:x} seg_file_end: {:x}\n\tfile_start: {:x} file_end: {:x} file_page_start: {:x} file_length: {:x}", seg_start, seg_end, seg_page_start, seg_page_end, seg_file_end, file_start, file_end, file_page_start, file_length); }

        if file_length != 0 {
            let mmap_flags = mmap::MAP_FIXED | mmap::MAP_PRIVATE;
            let prot_flags = pflags_to_prot(phdr.p_flags);
            unsafe {
                let start = mmap::mmap(seg_page_start as *const u64,
                                       file_length as usize,
                                       prot_flags,
                                       mmap_flags as c_int,
                                       fd.as_raw_fd() as c_int,
                                       file_page_start as usize);

                if start == mmap::MAP_FAILED {

                    return Err(format!("<dryad> loading phdrs for {} failed with errno {}, aborting execution", &soname, get_errno()))
                }
            }
        }

        // TODO: other more boring shit to do with zero'd out extra pages if too big, etc.
        //seg_file_end = page::page_end(seg_file_end);
    }

    // TODO: i believe we'll move calling the constructors to the relocation phase, once the dependency resolution has run and has flattened the list into a linear depends upon sequence
    // call constructors:

    /*
    if link_info.init != 0 {
        unsafe {
            println!("Calling constructor @ {:#x} for {}", link_info.init + load_bias, soname);
            let init: (fn() -> ()) = mem::transmute::<u64, (fn() -> ())>(link_info.init + load_bias);
            init();
        }
    }
    */
    //TODO: make this an optional
    let pltgot = if link_info.pltgot == 0 { 0 } else { link_info.pltgot + load_bias }; // musl doesn't have a PLTGOT, for example
    let gnu_hash = if link_info.gnu_hash == 0 { None } else { Some (GnuHash::new((link_info.gnu_hash + load_bias) as *const u32, symtab.len())) };

    let shared_object = SharedObject {
        name: strtab.get(link_info.soname),
        load_bias: load_bias,
        libs: libs,
        map_begin: start,
        map_end: end,
        // TODO: if we do not mmap the phdrs they come back broken and garbled since they get dropped by the compiler, as the slice's backing vec is de-alloc'd after this scope ends
        // not important since we don't use (and we might be able to just drop the phdrs completely since they shouldn't need to be used by the linking process
        phdrs: &[], // TODO: re-add program headers to this struct?
        dynamic: dynamic,
        symtab: symtab,
        strtab: strtab,
        relatab: relatab,
        pltrelatab: pltrelatab,
        pltgot: pltgot as *const u64,
        gnu_hash: gnu_hash,
        load_path: Some (load_path),
    };

    Ok (shared_object)
}
