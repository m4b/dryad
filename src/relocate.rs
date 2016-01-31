#![allow(unused_assignments)]

/// TODO: remove unused_assignments after validating with relacount, et. al

use std::slice;

use utils;
use binary::elf::rela;
use binary::elf::sym;
use binary::elf::dyn;

// TODO: consider moving this to linker.rs as only it uses it
pub unsafe fn get_relocations(bias: u64, dynamic: &[dyn::Dyn]) -> &[rela::Rela] {
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
    // TODO: validate relaent, using relacount
    let count = (relasz / relaent) as usize;
    slice::from_raw_parts(rela as *const rela::Rela, count)
}

pub fn relocate(bias:u64, relas: &[rela::Rela], symtab: &[sym::Sym], strtab: *const u8) {
    for rela in relas {
        let typ = rela::r_type(rela.r_info);
        let sym = rela::r_sym(rela.r_info); // index into the sym table
        let symbol = &symtab[sym as usize];
        let name = utils::str_at(strtab, symbol.st_name as isize);
        let reloc = (rela.r_offset + bias) as *mut u64;
        println!("relocating {}({:?}) with addend {:x} to {:x}",
                 name, reloc, rela.r_addend, (rela.r_addend + bias as i64));
        match typ {
            // B + A
            rela::R_X86_64_RELATIVE => {
                // set the relocations address to the load bias + the addend
                unsafe { *reloc = (rela.r_addend + bias as i64) as u64; }
            },
            // S
            rela::R_X86_64_GLOB_DAT => {
                // resolve symbol;
                // 1. start with exe, then next in needed, then next until symbol found
                // 2. use gnu_hash with symbol name to get sym info
            },
            // S + A
            rela::R_X86_64_64 => {
            }
            _ => ()
        }
    }
}
