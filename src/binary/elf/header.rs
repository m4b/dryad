use std::mem;
use std::fmt;

use utils::*;

pub const EHDR_SIZE: usize = 64;

pub const ET_NONE: u16 = 0; /* No file type */
pub const ET_REL: u16 = 1; /* Relocatable file */
pub const ET_EXEC: u16 = 2; /* Executable file */
pub const ET_DYN: u16 = 3; /* Shared object file */
pub const ET_CORE: u16 = 4; /* Core file */
pub const ET_NUM: u16 = 5; /* Number of defined types */

#[inline]
fn et_to_str(et: u16) -> &'static str {
    match et {
        ET_NONE => "NONE",
        ET_REL => "REL",
        ET_EXEC => "EXEC",
        ET_DYN => "DYN",
        ET_CORE => "CORE",
        ET_NUM => "NUM",
        _ => "UNKNOWN_ET"
    }
}

#[repr(C)]
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

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "e_ident: {:?} e_type: {} e_machine: 0x{:x} e_version: 0x{:x} e_entry: 0x{:x} e_phoff: 0x{:x} e_shoff: 0x{:x} e_flags: {:x} e_ehsize: {} e_phentsize: {} e_phnum: {} e_shentsize: {} e_shnum: {} e_shstrndx: {}",
               self.e_ident,
               et_to_str(self.e_type),
               self.e_machine,
               self.e_version,
               self.e_entry,
               self.e_phoff,
               self.e_shoff,
               self.e_flags,
               self.e_ehsize,
               self.e_phentsize,
               self.e_phnum,
               self.e_shentsize,
               self.e_shnum,
               self.e_shstrndx
               )
    }
}


// this is not unsafe because the header's size is encoded in the function
pub fn from_bytes<'a>(bytes: &'a[u8; EHDR_SIZE]) -> &'a Header {
    unsafe { mem::transmute(bytes) }
/*
    Header {
        e_ident: &bytes[0..15],
        e_type: 0,
        e_machine: 0,
        e_version: 0,
        e_entry: 0,
        e_phoff: 0,
        e_shoff: 0,
        e_flags: 0,
        e_ehsize: 0,
        e_phentsize: 0,
        e_phnum: 0,
        e_shentsize: 0,
        e_shnum: 0,
        e_shstrndx: 0,
    }
*/
}

pub unsafe fn unsafe_as_header<'a>(hdrp: *const u64) -> &'a Header {
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
