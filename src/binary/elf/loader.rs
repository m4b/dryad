use std::os::unix::io::RawFd;

use utils::mmap;
use binary::elf::program_header;

pub fn load_size (phdrs: &[program_header::ProgramHeader]) -> (usize, u64, u64) {
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
    return ((max_vaddr - min_vaddr) as usize, min_vaddr, max_vaddr);
}

pub fn reserve_address_space (phdrs: &[program_header::ProgramHeader]) -> Result <(), String> {

    let (size, max_vaddr, min_vaddr) = load_size (&phdrs);
    let mmap_flags = mmap::MAP_PRIVATE | mmap::MAP_ANONYMOUS;
    let start = unsafe { mmap::mmap(0 as *const u64, size, mmap::PROT_NONE, mmap_flags, -1, 0) };
    println!("Reserved {} bytes @ 0x{:x}", size, start);
//    let load_bias = start - addr;
    Ok(())
}

pub fn load (fd: RawFd, phdrs: &[program_header::ProgramHeader]) -> Result <(), String> {
    try!(reserve_address_space(phdrs));
    Ok(())
}
