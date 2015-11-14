#![allow(private_no_mangle_fns)]

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

trait DynamicArray {
    unsafe fn debug_print (&self);
}

impl DynamicArray for [Dyn] {
    unsafe fn debug_print(&self) {
        for dyn in self {
            dyn.debug_print();
        }
    }
}

#[no_mangle]
pub unsafe fn get_dynamic_array<'a>(bias:u64, phdrs: &'a [ProgramHeader]) -> Option<&'a [Dyn]> {
    for phdr in phdrs {
        if phdr.p_type == PT_DYNAMIC {
            let dynp = (phdr.p_vaddr + bias) as *const Dyn;
            let mut idx = 0;
            write(&"DYN FOUND\n");
            while (*(dynp.offset(idx))).d_tag != DT_NULL {
                write_u64(idx as u64, false);
                write(&" ");
                idx += 1;
            }
            write(&"\nFINAL ");
            write_u64(idx as u64, false);
            write(&"\n");
            return Some(slice::from_raw_parts(dynp, idx as usize));
        }
    }
    None
}

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
            unsafe {
                write(&"getting string at 0x");
                write_u64(strtab+dyn.d_val+bias, true);
                write(&" : ");
            }
            let string = string_from_strtab((strtab + dyn.d_val + bias) as *const u8);
            unsafe {
                write(string);
                write(&"\n");
            }
            needed.push(string);
        }
    }
    needed
}

pub unsafe fn print_needed(needed: Vec<&str>) {
    write(&"Needed: \n");
    for lib in needed {
        write(&lib);
    }
    write(&"\n");
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
