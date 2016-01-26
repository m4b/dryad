use std::fs::File;
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::os::raw::{c_int};

use utils::mmap;
use utils::page;
use binary::elf::header;
use binary::elf::program_header;
use binary::elf::dyn;
use binary::elf::image::SharedObject;

// TODO: add safe adds, there's ridiculous amounts of casting everywhere
fn map_fragment(fd: &File, base: u64, offset: u64, size: u64) -> Result<(u64, usize, *const u64), String> {
    let offset = base + offset;
    let page_min = page::page_start(offset);
    let end_offset = offset + size;
    let end_offset = end_offset + page::page_offset(offset);

    let map_size:usize = (end_offset - page_min) as usize;
    if map_size as u64 >= size {
        return Err (format!("<dryad> Error: file {:#?} has map_size >= size, aborting", fd))
    }

    let map_start = unsafe { mmap::mmap(0 as *const u64,
                                        map_size,
                                        mmap::PROT_READ,
                                        mmap::MAP_PRIVATE as c_int,
                                        fd.as_raw_fd() as c_int,
                                        page_min as usize) };

    if map_start == mmap::MAP_FAILED {

        Err (format!("<dryad> Error: map failed for {:#?}, aborting", fd))

    } else {

        let data = (map_start + page::page_offset(offset)) as *const u64;
        Ok ((map_start, map_size, data))
    }
}

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
    println!("size: {:x} min_vaddr: {:x} max_vaddr: {:x}", max_vaddr - min_vaddr, min_vaddr, max_vaddr);
    ((max_vaddr - min_vaddr) as usize, min_vaddr, max_vaddr)
}

fn reserve_address_space (phdrs: &[program_header::ProgramHeader]) -> Result <(u64, u64), String> {

    let (size, min_vaddr, max_vaddr) = compute_load_size(&phdrs);

    let mmap_flags = mmap::MAP_PRIVATE | mmap::MAP_ANONYMOUS;
    let start = unsafe { mmap::mmap(0 as *const u64,
                                    size,
                                    mmap::PROT_NONE,
                                    mmap_flags as c_int,
                                    -1,
                                    0) };

    if start == mmap::MAP_FAILED {

        Err(format!("<dryad> Failure: anonymous mmap failed for size {:x}", size))

    } else {

        let load_bias = start - min_vaddr;
        println!("Reserved {:x} bytes @ 0x{:x} with bias {:x}", size, start, load_bias);

        Ok((start, load_bias))
    }
}

#[inline(always)]
fn pflags_to_prot (x:u32) -> isize {
    use binary::elf::program_header::{PF_X, PF_R, PF_W};

    (if x & PF_X == PF_X { mmap::PROT_EXEC } else { 0 }) |
    (if x & PF_R == PF_R { mmap::PROT_READ } else { 0 }) |
    (if x & PF_W == PF_W { mmap::PROT_WRITE } else { 0 })
}

extern {
    /// musl #defines erro *(__errno_location()) ... so errno isn't a symbol in the final binary and accesses will segfault us. yay.
    fn __errno_location() -> *const i32;
}

/// Loads an ELF binary from the given fd, mmaps its contents, and returns a SharedObject
/// TODO: probably just move this function to image and use it as the impl
pub fn load<'a> (soname: &str, fd: &mut File) -> Result <SharedObject, String> {

    // 1. Suck up the elf header and construct the program headers
    let mut elf_header = [0; header::EHDR_SIZE];
    let _ = fd.read(&mut elf_header);

    let elf_header = header::from_bytes(&elf_header);
    // TODO: phdr should be mmapped and not copied?
    let mut phdrs: Vec<u8> = vec![0; (elf_header.e_phnum as u64 * program_header::PHDR_SIZE) as usize];
    let _ = fd.read(phdrs.as_mut_slice());
    // TODO: ditto, experiment with mmap vs malloc'ing into memory and using vecs
    let phdrs = program_header::from_bytes(&phdrs, elf_header.e_phnum as usize);
    println!("header:\n  {:#?}\nphdrs:\n  {:#?}", &elf_header, &phdrs);

    // 1.5 mmap the dynamic array with the strtab so we can access them and resolve symbol lookups against this library; this will require mmapping the segments, and storing the dynamic array, along with the strtab; or why not just suck them up ourselves into memory and resolve queries against it that way -- probably slower...

    // this is redundant, use the link info shite
    for phdr in phdrs {
    }
//    let (dynamic_start, dynamic_size, dynamic_data) = map_fragment(fd,

    // 2. Reserve address space with anon mmap
    let (start, load_bias) = try!(reserve_address_space(&phdrs));

    //TODO: figure out what the file offset, if any, should be
    let file_offset:usize = 0;

    // 3. mmap the PT_LOAD program headers
    for phdr in phdrs {

        if phdr.p_type != program_header::PT_LOAD {
            continue
        }

        let seg_start:u64 = phdr.p_vaddr + load_bias;
        let seg_end:u64   = seg_start + phdr.p_memsz;

        let seg_page_start:u64 = page::page_start(seg_start);
        let seg_page_end:u64   = page::page_start(seg_end);

        let mut seg_file_end:u64 = seg_start + phdr.p_filesz;

        // File offsets.
        let file_start:u64 = phdr.p_offset;
        let file_end:u64   = file_start + phdr.p_filesz;

        let file_page_start = page::page_start(file_start);
        let file_length:u64 = file_end - file_page_start;

        // TODO: add error checking, if file size <= 0, if file_end greater than file_size, etc.

        println!("DATA:\n\tseg_start: {:x} seg_end: {:x} seg_page_start: {:x} seg_page_end: {:x} seg_file_end: {:x}\n\tfile_start: {:x} file_end: {:x} file_page_start: {:x} file_length: {:x}", seg_start, seg_end, seg_page_start, seg_page_end, seg_file_end, file_start, file_end, file_page_start, file_length);

        if file_length != 0 {
            let mmap_flags = mmap::MAP_FIXED | mmap::MAP_PRIVATE;
            let prot_flags = pflags_to_prot(phdr.p_flags);
            unsafe {
                let start = mmap::mmap(seg_page_start as *const u64,
                                       file_length as usize,
                                       prot_flags,
                                       mmap_flags as c_int,
                                       fd.as_raw_fd() as c_int,
                                       file_offset + file_page_start as usize);
                if start == mmap::MAP_FAILED {
                    return Err(format!("<dryad> loading phdrs for {} failed with errno {}, aborting execution", &soname, *__errno_location()))
                }
            }
        }

        seg_file_end = page::page_end(seg_file_end);
    }

    // TODO: move this above, for early termination...
    // 4. load the dynamic array
    if let Some(dynamic) = unsafe { dyn::get_dynamic_array(load_bias, &phdrs) } {
        println!("LOAD: header:\n  {:#?}\nphdrs:\n  {:#?}\ndynamic:\n  {:#?}", &elf_header, &phdrs, &dynamic);

        // TODO: get libs dyn::get_needed()

        let shared_object = SharedObject {
            name: soname.to_string(),
            phdrs: phdrs.to_owned(),
            dynamic: dynamic.to_owned(),
            base: 0,
            load_bias: load_bias,
            // TODO: don't forget this
            libs: vec!["Hi".to_string()],
        };

        Ok(shared_object)

    } else {
        Err(format!("<dryad> no dynamic array found for {}", &soname))
    }
}
