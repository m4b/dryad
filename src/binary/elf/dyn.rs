#![allow(private_no_mangle_fns)]

use std::str;
use std::slice;
use utils::*;
use binary::elf::program_header::ProgramHeader;

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
pub unsafe fn get_dynamic_array<'a>(phdrs: &'a [ProgramHeader]) -> Option<&'a [Dyn]> {
    for phdr in phdrs {
//        write_u64(phdr.p_type as u64, false);
        if phdr.p_type == 2 { // TODO: PT::DYNAMIC = PT_DYNAMIC = 2
            let dynp = phdr.p_vaddr as *const Dyn;
            let mut idx = 0;
            write_u64(idx as u64, false);
            write(&"DYN FOUND\n");
            while (*(dynp.offset(idx))).d_tag != 0 {
                idx += 1;
            }
            return Some(slice::from_raw_parts(dynp, idx as usize));
        }
    }
    None
}

pub fn get_strtab(dyns: &[Dyn]) -> u64 {
    for dyn in dyns {
        match dyn.d_tag {
            5 => return dyn.d_val,
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

pub fn get_needed<'a>(dyns: &'a [Dyn], strtab: u64, base: u64) -> Vec<&'a str> {
    let mut needed = vec![];
    for dyn in dyns {
        if dyn.d_tag == 1 { // TODO: DT::NEEDED = DT_NEEDED = 1
            let string = string_from_strtab((strtab + base +             dyn.d_val) as *const u8);
            needed.push(string);
        }
    }
    return needed;
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
