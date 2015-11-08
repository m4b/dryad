use utils::*;
use std::mem;

#[repr(C)]
#[derive(Debug)]
pub struct Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

pub unsafe fn as_header<'a>(hdrp: *const u64) -> &'a Header {
    mem::transmute(hdrp)
        //        let e_ident = slice::from_raw_parts(hdrp, 16);
        //        let e_type = 
        //        Header{  }
}

impl Header {    
    pub unsafe fn debug_print (&self) {
        write(&"-=Elf64_hdr=-\n");
        write(&"e_type: 0x");
        write_u64(self.e_type as u64, true);
        write(&"\n");
        write(&"e_machine: 0x");
        write_u64(self.e_machine as u64, true);
        write(&"\n");
        write(&"e_version: 0x");
        write_u64(self.e_version as u64, true);
        write(&"\n");
        write(&"e_entry: 0x");
        write_u64(self.e_entry, true);
        write(&"\n");
        write(&"e_phoff: 0x");
        write_u64(self.e_phoff, true);
        write(&"\n");
        write(&"e_shoff: 0x");
        write_u64(self.e_shoff, true);
        write(&"\n");
        write(&"e_flags: 0x");
        write_u64(self.e_flags as u64, true);
        write(&"\n");
        write(&"e_ehsize: 0x");
        write_u64(self.e_ehsize as u64, true);
        write(&"\n");
        write(&"e_phentsize: 0x");
        write_u64(self.e_phentsize as u64, true);
        write(&"\n");
        write(&"e_phnum: 0x");
        write_u64(self.e_phnum as u64, true);
        write(&"\n");
        write(&"e_shentsize: 0x");
        write_u64(self.e_shentsize as u64, true);
        write(&"\n");
        write(&"e_shnum: 0x");
        write_u64(self.e_shnum as u64, true);
        write(&"\n");
        write(&"e_shstrndx: 0x");
        write_u64(self.e_shstrndx as u64, true);
        write(&"\n");
    }
}
