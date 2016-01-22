use std::os::unix::io::RawFd;

use utils::mmap;
use utils::page;
use binary::elf::program_header;

fn load_size (phdrs: &[program_header::ProgramHeader]) -> (usize, u64, u64) {
    let mut max_vaddr = 0;
    let mut min_vaddr = 0;
    for phdr in phdrs {
        if phdr.p_type != program_header::PT_LOAD {
            continue;
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
    return ((max_vaddr - min_vaddr) as usize, min_vaddr, max_vaddr);
}

fn reserve_address_space (phdrs: &[program_header::ProgramHeader]) -> Result <(u64, u64), String> {

    let (size, min_vaddr, max_vaddr) = load_size(&phdrs);

    let mmap_flags = mmap::MAP_PRIVATE | mmap::MAP_ANONYMOUS;
    let start = unsafe { mmap::mmap(0 as *const u64,
                                    size,
                                    mmap::PROT_NONE,
                                    mmap_flags,
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

pub fn load (soname: &str, fd: RawFd, phdrs: &[program_header::ProgramHeader]) -> Result <(), String> {
    let (start, load_bias) = try!(reserve_address_space(phdrs));

    //TODO: figure out what the file offset, if any, should be
    let file_offset:usize = 0;

    for phdr in phdrs {

        if phdr.p_type != program_header::PT_LOAD {
            continue;
        }

        let seg_start:u64 = phdr.p_vaddr + load_bias;
        let seg_end:u64   = seg_start + phdr.p_memsz;

        let seg_page_start:u64 = page::page_start(seg_start);
        let seg_page_end:u64   = page::page_start(seg_end);

        let mut seg_file_end:u64   = seg_start + phdr.p_filesz;

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
                                       mmap_flags,
                                       fd as isize,
                                       file_offset + file_page_start as usize);
                if start == mmap::MAP_FAILED {
                    return Err(format!("<dryad> loading phdrs for {} failed with errno {}, aborting execution", &soname, *__errno_location()));
                }
            }
        }

        seg_file_end = page::page_end(seg_file_end);
    }

    Ok(())
}
