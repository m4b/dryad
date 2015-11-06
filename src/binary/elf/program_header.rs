use std::slice;
use utils::*;

#[repr(C)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

impl ProgramHeader {
    pub unsafe fn debug_print (&self) {
        write(&"-==Elf64_phdr==-\n");
        write(&"p_type: 0x");
        write_u64(self.p_type as u64, true);
        write(&"\n");
        write(&"p_flags: 0x");
        write_u64(self.p_flags as u64, true);
        write(&"\n");
        write(&"p_offset: 0x");
        write_u64(self.p_offset, true);
        write(&"\n");
        write(&"p_vaddr: 0x");
        write_u64(self.p_vaddr, true);
        write(&"\n");
        write(&"p_paddr: 0x");
        write_u64(self.p_paddr, true);
        write(&"\n");
        write(&"p_filesz: 0x");
        write_u64(self.p_filesz as u64, true);
        write(&"\n");
        write(&"p_memsz: 0x");
        write_u64(self.p_filesz as u64, true);
        write(&"\n");
        write(&"p_align: ");
        write_u64(self.p_align as u64, false);
        write(&"\n");
    }
    pub fn size_of(&self) -> u64 {
        64
    }
}

pub unsafe fn to_phdr_array<'a>(phdrp: *const ProgramHeader, phnum: usize) -> &'a[ProgramHeader] {
    slice::from_raw_parts(phdrp, phnum)
}

pub unsafe fn debug_print_phdrs (phdrs: &[ProgramHeader]) {
    for phdr in phdrs {
        phdr.debug_print();
    }
}
