use std::fmt;
use std::str;
use std::slice;
use utils::*;
use binary::elf::program_header::*;

#[repr(C)]
pub struct Dyn {
    pub d_tag: u64, // Dynamic entry type
    pub d_val: u64, // Integer value
}

impl Dyn {
    unsafe fn debug_print(&self) {
        write(&"d_tag: 0x");
        write_u64(self.d_tag, true);
        write(&"\n");
        write(&"d_val: 0x");
        write_u64(self.d_val, true);
        write(&"\n");
    }
}

#[inline]
fn tag_to_str(tag:u64) -> &'static str {
    match tag {
        0 => "DT_NULL",
        1 => "DT_NEEDED",
        2 => "DT_PLTRELSZ",
        3 => "DT_PLTGOT",
        4 => "DT_HASH",
        5 => "DT_STRTAB",
        6 => "DT_SYMTAB",
        7 => "DT_RELA",
        8 => "DT_RELASZ",
        9 => "DT_RELAENT",
        10 => "DT_STRSZ:",
        11 => "DT_SYMENT",
        12 => "DT_INIT",
        13 => "DT_FINI",
        14 => "DT_SONAME",
        15 => "DT_RPATH",
        16 => "DT_SYMBOLIC",
        17 => "DT_REL",
        18 => "DT_RELSZ",
        19 => "DT_RELENT",
        20 => "DT_PLTREL",
        21 => "DT_DEBUG",
        22 => "DT_TEXTREL",
        23 => "DT_JMPREL",
        24 => "DT_BIND_NOW",
        25 => "DT_INIT_ARRAY",
        26 => "DT_FINI_ARRAY",
        27 => "DT_INIT_ARRAYSZ",
        28 => "DT_FINI_ARRAYSZ",
        29 => "DT_RUNPATH",
        30 => "DT_FLAGS",
        32 => "DT_PREINIT_ARRAY",
        33 => "DT_PREINIT_ARRAYSZ",
        34 => "DT_NUM",
        0x6000000d => "DT_LOOS",
        0x6ffff000 => "DT_HIOS",
        0x70000000 => "DT_LOPROC",
        0x7fffffff => "DT_HIPROC",
        0x6ffffff0 => "DT_VERSYM",
        0x6ffffff9 => "DT_RELACOUNT",
        0x6ffffffa => "DT_RELCOUNT",
        // new
        0x6ffffef5 => "DT_GNU_HASH",
        0x6ffffffc => "DT_VERDEF",
        0x6ffffffd => "DT_VERDEFNUM",
        0x6ffffffe => "DT_VERNEED",
        0x6fffffff => "DT_VERNEEDNUM",
        _ => "UNKNOWN_TAG"
    }
}

impl fmt::Debug for Dyn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "d_tag: {} d_val: 0x{:x}", tag_to_str(self.d_tag), self.d_val)
    }
}

pub unsafe fn get_dynamic_array<'a>(bias:u64, phdrs: &'a [ProgramHeader]) -> Option<&'a [Dyn]> {
    for phdr in phdrs {
        if phdr.p_type == PT_DYNAMIC {
            let dynp = (phdr.p_vaddr + bias) as *const Dyn;
            let mut idx = 0;
            while (*(dynp.offset(idx))).d_tag != DT_NULL {
                idx += 1;
            }
            return Some(slice::from_raw_parts(dynp, idx as usize));
        }
    }
    None
}

// TODO: have this return a &[u8] since we know the size of the strtab from dyn
pub fn get_strtab(bias:u64, dyns: &[Dyn]) -> u64 {
    for dyn in dyns {
        match dyn.d_tag {
            DT_STRTAB => return dyn.d_val + bias,
            _ => (),
        }
    }
    0
}

pub fn string_from_strtab<'a> (offset: *const u8) -> &'a str {
    let mut i = 0;
    unsafe {
        while *offset.offset(i) != 0 {
            i += 1;
        }
        let slice = slice::from_raw_parts(offset, i as usize);
        str::from_utf8(slice).unwrap()
    }
}

pub fn get_needed<'a>(dyns: &'a [Dyn], strtab: u64, base: u64, bias: u64) -> Vec<&'a str> {
    let mut needed = vec![];
    for dyn in dyns {
        if dyn.d_tag == DT_NEEDED {
            let string = string_from_strtab((strtab + dyn.d_val + bias) as *const u8);
            needed.push(string);
        }
    }
    needed
}

pub unsafe fn debug_print_dynamic(dynamic: &[Dyn]) {
    for dyn in dynamic {
        dyn.debug_print();
    }
}


/*
 CONSTS
*/

pub const DT_NULL:u64 = 0;
pub const DT_NEEDED:u64 = 1;
pub const DT_PLTRELSZ:u64 = 2;
pub const DT_PLTGOT:u64 = 3;
pub const DT_HASH:u64 = 4;
pub const DT_STRTAB:u64 = 5;
pub const DT_SYMTAB:u64 = 6;
pub const DT_RELA:u64 = 7;
pub const DT_RELASZ:u64 = 8;
pub const DT_RELAENT:u64 = 9;
pub const DT_STRSZ:u64 = 10;
pub const DT_SYMENT:u64 = 11;
pub const DT_INIT:u64 = 12;
pub const DT_FINI:u64 = 13;
pub const DT_SONAME:u64 = 14;
pub const DT_RPATH:u64 = 15;
pub const DT_SYMBOLIC:u64 = 16;
pub const DT_REL:u64 = 17;
pub const DT_RELSZ:u64 = 18;
pub const DT_RELENT:u64 = 19;
pub const DT_PLTREL:u64 = 20;
pub const DT_DEBUG:u64 = 21;
pub const DT_TEXTREL:u64 = 22;
pub const DT_JMPREL:u64 = 23;
pub const DT_BIND_NOW:u64 = 24;
pub const DT_INIT_ARRAY:u64 = 25;
pub const DT_FINI_ARRAY:u64 = 26;
pub const DT_INIT_ARRAYSZ:u64 = 27;
pub const DT_FINI_ARRAYSZ:u64 = 28;
pub const DT_RUNPATH:u64 = 29;
pub const DT_FLAGS:u64 = 30;
pub const DT_ENCODING:u64 = 32;
pub const DT_PREINIT_ARRAY:u32 = 32;
pub const DT_PREINIT_ARRAYSZ:u32 = 33;
pub const DT_NUM:u64 = 34;
pub const DT_LOOS:u64 = 0x6000000d;
pub const DT_HIOS:u64 = 0x6ffff000;
pub const DT_LOPROC:u64 = 0x70000000;
pub const DT_HIPROC:u64 = 0x7fffffff;
//pub const DT_PROCNUM:u64 = DT_MIPS_NUM;
pub const DT_VERSYM:u64 = 0x6ffffff0;
pub const DT_RELACOUNT:u64 = 0x6ffffff9;
pub const DT_RELCOUNT:u64 = 0x6ffffffa;

// new
pub const DT_GNU_HASH:u64 = 0x6ffffef5;
pub const DT_VERDEF:u64 = 0x6ffffffc;
pub const DT_VERDEFNUM:u64 = 0x6ffffffd;
pub const DT_VERNEED:u64 = 0x6ffffffe;
pub const DT_VERNEEDNUM:u64 = 0x6fffffff;
