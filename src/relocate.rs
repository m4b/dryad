#![allow(unused_assignments)]

use std::slice;

//use utils::*;
use binary::elf::rela;
use binary::elf::sym;
use binary::elf::dyn;

// TODO: remove the load bias, or this function altogether
pub unsafe fn get_relocations(bias: u64, dynamic: &[dyn::Dyn]) -> &[rela::Elf64_Rela] {
    let mut rela = 0;
    let mut relasz = 0;
    let mut relaent = 0;
    let mut relacount = 0;
    for dyn in dynamic {
        match dyn.d_tag {
            dyn::DT_RELA => {rela = dyn.d_val + bias;},
            dyn::DT_RELASZ => {relasz = dyn.d_val;},
            dyn::DT_RELAENT => {relaent = dyn.d_val;},
            dyn::DT_RELACOUNT => {relacount = dyn.d_val;},
            _ => ()
        }
    }
    // TODO: validate relaent,
    let count = (relasz / relaent) as usize;
    slice::from_raw_parts(rela as *const rela::Elf64_Rela, count)
}

pub unsafe fn relocate(bias:u64, relas: &[rela::Elf64_Rela], symtab: &[sym::Sym]) {
    for rela in relas {
        let typ = rela::r_type(rela.r_info);
        let sym = rela::r_sym(rela.r_info); // index into the sym table
        let symbol = &symtab[sym as usize];
        let reloc = rela.r_offset + bias;
        match typ {
            rela::R_X86_64_RELATIVE => {
                let addr = reloc as *mut u64;
                /*
                write(&"relocating addr 0x");
                write_u64(rela.r_offset, true);
                write(&" with addend 0x");
                write_u64(rela.r_addend as u64, true);
                write(&" to 0x");
                write_u64((rela.r_addend + bias as i64) as u64, true);
                write(&"\n");
                 */
                // set the relocations address to the load bias + the addend
                *addr = (rela.r_addend + bias as i64) as u64;
            },
            _ => ()
        }
    }
}
