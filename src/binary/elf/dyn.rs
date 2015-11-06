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

pub unsafe fn get_dynamic_array<'a>(phdrs: &'a [ProgramHeader]) -> Option<&'a [Dyn]> {
    for phdr in phdrs {
        if phdr.p_type == 2 { // TODO: PT::DYNAMIC = PT_DYNAMIC = 2
            let dynp = phdr.p_vaddr as *const Dyn;
            let mut idx = 0;
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


// first we need to relocate ourselves (the linker), then we can
// do relocation and symbol binding the easy way using heap allocations
/*
pub fn get_needed<'a>(dyns: &'a [Dyn], strtab: u64, base: u64) -> Option<&'a [&str]> {
    let mut needed = [""; 30];
    for dyn in dyns {
        if dyn.dt_type == 1 { // TODO: DT::NEEDED = DT_NEEDED = 1
            
        }
    }
    for i in 0..count {
        needed[i] = 
    }
    return Some(slice::from_raw_parts(dynp, idx as usize));
}
*/

pub unsafe fn debug_print_dynamic(dynamic: &[Dyn]) {
    for dyn in dynamic {
        dyn.debug_print();
    }
}
