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

pub struct Elf {
    pub header: Header,
//    pub program_headers: [ElfPhdr],
}

impl Header {
    pub fn print_debug(&self) {
        
    }
}

/*
impl Elf {
    pub fn new(block:kernel_block::KernelBlock) {
        
    }
}
*/
