/// TODO: parse and return flags per DSO, add as entry to the struct
/// TODO: fix the high address mapperr for `__libc_start_main`

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::os::raw::{c_int};

use utils::{self, mmap, page};
use image::SharedObject;
use binary::elf::header;
use binary::elf::program_header;
use binary::elf::dyn;
use binary::elf::sym;
use binary::elf::rela;
use binary::elf::strtab::Strtab;
use binary::elf::gnu_hash::GnuHash;

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
                                    // for now, using PROT_NONE seems to give SEGV_ACCERR on execution of PT_LOAD mmaped segments (i.e., operation not allowed on mapped object)
                                    mmap::PROT_EXEC | mmap::PROT_READ | mmap::PROT_WRITE,
                                    mmap_flags as c_int,  // TODO: I think we should copy glibc's lead here and use their mmap flags
                                    -1,
                                    0) };

    if start == mmap::MAP_FAILED {

        Err (format!("<dryad> Failure: anonymous mmap failed for size {:x} with errno {}", size, utils::get_errno()))

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

/// Loads an ELF binary from the given fd, mmaps its contents, and returns a SharedObject, whose lifetime is tied to the mmap's, i.e., manually managed
pub fn load<'a> (soname: &str, load_path: String, fd: &mut File, debug: bool) -> Result <SharedObject<'a>, String> {

    ///////////////
    // Part I:
    //   wherein we read the binary from disk,
    //   and lovingly mmap it's joyous contents
    ///////////////

    // 1. Suck up the elf header on disk and construct the program headers
    let ehdr = header::Header::from_fd(fd).map_err(|e| format!("<dryad> Error {:?}", e))?;
    let phdrs = program_header::ProgramHeader::from_fd(fd, ehdr.e_phoff, ehdr.e_phnum as usize).map_err(|e| format!("<dryad> Error {:?}", e))?;

    // 2. Reserve address space with anon mmap
    let (start, load_bias, end) = reserve_address_space(&phdrs)?;
    dbgc!(red_bold: debug, "loader", "reserved {:#x} - {:#x} with load_bias: 0x{:x}", start, end, load_bias);

    // 3. Now we iterate through the phdrs, and
    // a. mmap the PT_LOAD program headers
    // b. collect the vaddrs of the phdrs and the dynamic array
    // c. TODO: mmap and setup TLS
    let mut phdrs_vaddr = 0;
    let mut dynamic_vaddr = None;
    let mut has_pt_load = false;
    for phdr in &phdrs {

        match phdr.p_type {

            program_header::PT_LOAD => {
                has_pt_load = true;
                // Segment offsets: rounds down the segment start to a value suitable for mmaping, and adjusts the size of the 
                // mmap breadth appropriately
                let seg_start: u64 = phdr.p_vaddr + load_bias;
                let seg_end: u64   = seg_start + phdr.p_memsz;

                let seg_page_start: u64 = page::page_start(seg_start);
                let seg_page_end: u64   = page::page_start(seg_end);

                // TODO: unused, I think we need to zero some stuff
                let seg_file_end: u64 = seg_start + phdr.p_filesz;

                // File offsets.
                let file_start: u64 = phdr.p_offset;
                let file_end: u64   = file_start + phdr.p_filesz;

                // "rounds" to an mmap-able value (i.e., file_start % pagesize)
                // file_page_start <= file_start
                // so sometimes the beginning of the page is not the beginning of the PT_LOAD!
                let file_page_start = page::page_start(file_start);
                let file_length: u64 = file_end - file_page_start;

                // TODO: add error checking, if file size <= 0, if file_end greater than file_size, etc.

                dbgc!(red_bold: debug, "loader", "PT_LOAD:\n\tseg_start: {:x} seg_end: {:x} seg_page_start: {:x} seg_page_end: {:x} seg_file_end: {:x}\n\tfile_start: {:x} file_end: {:x} file_page_start: {:x} file_length: {:x}", seg_start, seg_end, seg_page_start, seg_page_end, seg_file_end, file_start, file_end, file_page_start, file_length);

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

                            return Err(format!("loader loading phdrs for {} failed with errno {}, aborting execution", &soname, utils::get_errno()))
                        }
                    }

                    // TODO: other more boring shit to do with zero'd out extra pages if too big, etc.
                    //seg_file_end = page::page_end(seg_file_end);

                }
            } // end match PT_LOAD

            program_header::PT_PHDR => {
                phdrs_vaddr = phdr.p_vaddr;
            },
            program_header::PT_DYNAMIC => {
                dynamic_vaddr = Some(phdr.p_vaddr);
            },
            _ => () // do nothing, i.e., continue
        }
    }

    if !has_pt_load {
        return Err(format!("loader {} has no PT_LOAD sections", soname));
    }

    ///////////////
    // Part Deux:
    //   wherein we construct our components for this SharedObject
    //   from the newly mmap'd memory
    ///////////////

    // use the now mmap'd program headers
    let phdrs = unsafe { program_header::ProgramHeader::from_raw_parts((phdrs_vaddr + load_bias) as *const program_header::ProgramHeader, phdrs.len()) };

    // construct the dynamic slice in whatever mmap'd PT_LOAD it's in
    let dynamic_vaddr = dynamic_vaddr.ok_or(format!("loader {} has no dynamic array", soname))?;
    let dynamic = unsafe { dyn::from_raw(load_bias, dynamic_vaddr) };

    // build the link info with the bias and the dynamic array
    let link_info = dyn::LinkInfo::new(&dynamic, load_bias as usize);

    // now get the strtab from the dynamic array
    let strtab = unsafe { Strtab::from_raw(link_info.strtab as *const u8, link_info.strsz as usize) };

    // now get the libs we will need
    let libs = dyn::get_needed(dynamic, &strtab, link_info.needed_count);

    // caveat about rdr doing this for hundres of binaries and it being "ok"
    let num_syms = (link_info.strtab - link_info.symtab) / sym::SIZEOF_SYM;

    // now construct the symtab
    let symtab = unsafe { sym::from_raw(link_info.symtab as *const sym::Sym, num_syms) };

    // now grab relatab, and pltreltab which we'll use for relocating this shared object later
    let relatab = unsafe { rela::from_raw(link_info.rela as *const rela::Rela, link_info.relasz) };
    let pltrelatab = unsafe { rela::from_raw(link_info.jmprel as *const rela::Rela, link_info.pltrelsz) };

    // the pltgot we need for doing lazy dynamic linking
    let pltgot = if let Some(addr) = link_info.pltgot { addr } else { 0 }; // musl doesn't have a PLTGOT, for example

    // and finally grab the gnu_hash (if it has one)
    let gnu_hash = if let Some(addr) = link_info.gnu_hash { Some (GnuHash::new(addr as *const u32, symtab.len())) } else { None };

    let shared_object = SharedObject {
        name: strtab.get(link_info.soname),
        load_bias: load_bias,
        libs: libs,
        map_begin: start,
        map_end: end,
        phdrs: phdrs,
        dynamic: dynamic,
        symtab: symtab,
        strtab: strtab,
        relatab: relatab,
        pltrelatab: pltrelatab,
        pltgot: pltgot as *const u64,
        gnu_hash: gnu_hash,
        load_path: Some (load_path),
        flags: link_info.flags,
        state_flags: link_info.flags_1,
    };

    Ok (shared_object)
}
