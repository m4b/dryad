use std::slice;
use std::fmt;
use utils::*;

pub const PHDR_SIZE:u64 = 64;

pub const PT_NULL:u32 = 0;
pub const PT_LOAD:u32 = 1;
pub const PT_DYNAMIC:u32 = 2;
pub const PT_INTERP:u32 =	3;
pub const PT_NOTE:u32 = 4;
pub const PT_SHLIB:u32 = 5;
pub const PT_PHDR:u32 = 6;
pub const PT_TLS:u32 = 7;
pub const PT_NUM:u32 = 8;
pub const PT_LOOS:u32 = 0x60000000;
pub const PT_GNU_EH_FRAME:u32 = 0x6474e550;
pub const PT_GNU_STACK:u32 = 0x6474e551;
pub const PT_GNU_RELRO:u32 = 0x6474e552;
pub const PT_LOSUNW:u32 = 0x6ffffffa;
pub const PT_SUNWBSS:u32 = 0x6ffffffa;
pub const PT_SUNWSTACK:u32 = 0x6ffffffb;
pub const PT_HISUNW:u32 = 0x6fffffff;
pub const PT_HIOS:u32 = 0x6fffffff;
pub const PT_LOPROC:u32 = 0x70000000;
pub const PT_HIPROC:u32 = 0x7fffffff;

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

impl fmt::Debug for ProgramHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "p_type: 0x{:x} p_flags 0x{:x} p_offset: 0x{:x} p_vaddr: 0x{:x} p_vaddr: 0x{:x} p_filesz: 0x{:x} p_memsz: 0x{:x} p_align: {}",
               self.p_type, self.p_flags, self.p_offset, self.p_vaddr,
               self.p_paddr, self.p_filesz, self.p_memsz, self.p_align
               )
    }
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
}

pub unsafe fn to_phdr_array<'a>(phdrp: *const ProgramHeader, phnum: usize) -> &'a[ProgramHeader] {
    slice::from_raw_parts(phdrp, phnum)
}

pub unsafe fn debug_print_phdrs (phdrs: &[ProgramHeader]) {
    for phdr in phdrs {
        phdr.debug_print();
    }
}
