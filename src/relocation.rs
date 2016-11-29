use std::slice;

use elf::{program_header, dyn, reloc};
use utils;

#[cfg(target_arch = "x86_64")]
pub const DTPMOD64: u32 = reloc::R_X86_64_DTPMOD64;
#[cfg(target_arch = "x86_64")]
pub const RELATIVE: u32 = reloc::R_X86_64_RELATIVE;
#[cfg(target_arch = "x86_64")]
pub const IRELATIVE: u32 = reloc::R_X86_64_IRELATIVE;
#[cfg(target_arch = "x86_64")]
pub const JUMP_SLOT: u32 = reloc::R_X86_64_JUMP_SLOT;
#[cfg(target_arch = "x86_64")]
pub const GLOB_DAT: u32 = reloc::R_X86_64_GLOB_DAT;

#[cfg(target_arch = "aarch64")]
pub const DTPMOD64: u32 = reloc::R_AARCH64_TLS_DTPMOD;
#[cfg(target_arch = "aarch64")]
pub const DTPMOD32: u32 = reloc::R_AARCH64_P32_TLS_DTPMOD;
#[cfg(target_arch = "aarch64")]
pub const JUMP_SLOT: u32 = reloc::R_AARCH64_JUMP_SLOT;
#[cfg(target_arch = "aarch64")]
pub const GLOB_DAT: u32 = reloc::R_AARCH64_GLOB_DAT;
#[cfg(target_arch = "aarch64")]
pub const RELATIVE: u32 = reloc::R_AARCH64_RELATIVE;
#[cfg(target_arch = "aarch64")]
pub const IRELATIVE: u32 = reloc::R_AARCH64_IRELATIVE;

#[cfg(target_arch = "arm")]
pub const DTPMOD32: u32 = reloc::R_ARM_TLS_DTPMOD32;
#[cfg(target_arch = "arm")]
pub const JUMP_SLOT: u32 = reloc::R_ARM_JUMP_SLOT;
#[cfg(target_arch = "arm")]
pub const GLOB_DAT: u32 = reloc::R_ARM_GLOB_DAT;
#[cfg(target_arch = "arm")]
pub const RELATIVE: u32 = reloc::R_ARM2_RELATIVE;
#[cfg(target_arch = "arm")]
pub const IRELATIVE: u32 = reloc::R_ARM_IRELATIVE;

#[cfg(target_arch = "x86")]
pub const JUMP_SLOT: u32 = reloc::R_386_JMP_SLOT;
#[cfg(target_arch = "x86")]
pub const GLOB_DAT: u32 = reloc::R_386_GLOB_DAT;
#[cfg(target_arch = "x86")]
pub const RELATIVE: u32 = reloc::R_386_RELATIVE;
#[cfg(target_arch = "x86")]
pub const IRELATIVE: u32 = reloc::R_386_IRELATIVE;

macro_rules! addend {
    ($reloc:ident, $addr:ident) => {
        #[cfg(target_pointer_width = "64")]
        reloc.r_addend as isize
        #[cfg(target_pointer_width = "32")]
        (*addr) as isize
    }
}

unsafe fn get_linker_relocations(info: &dyn::DynamicInfo) -> (&[reloc::Rela], &[reloc::Rel]) {
    let rela =
        if info.relaent == 0 {
        &[]
    } else {
        let count = info.relasz / info.relaent as usize;
        slice::from_raw_parts(info.rela as *const reloc::Rela, count)
    };
    let rel =
        if info.relent == 0 {
        &[]
    } else {
        let count = info.relsz / info.relent as usize;
        slice::from_raw_parts(info.rel as *const reloc::Rel, count)
    };
    (rela, rel)
}

/// TODO: i think this is false; we may need to relocate R_X86_64_GLOB_DAT and R_X86_64_64
/// DTPMOD64 is showing up in relocs if we make dryad -shared instead of -pie.  and this is because it leaves local executable TLS model because the damn hash map uses random TLS data.  `working_set` has been the bane of my life in this project
/// private linker relocation function; assumes dryad _only_
/// contains X86_64_RELATIVE relocations, which should be true
pub fn relocate_linker(bias: usize, info: &dyn::DynamicInfo, phdrs: &[program_header::ProgramHeader]) {
    let (relas, rels) = unsafe { get_linker_relocations(&info)};

    if info.textrel {
        // TODO: fail here, need to add custom error code
        let res = utils::mmap::mprotect_phdrs(phdrs, bias, utils::mmap::PROT_WRITE);
    }
    #[cfg(target_pointer_width = "32")]
    for rel in rels {
        if reloc::r_type(rel.r_info) == RELATIVE {
            let reloc = (rel.r_offset as usize + bias) as *mut usize;
            unsafe {
                *reloc = bias + *reloc;
            }
        }
    }
    #[cfg(target_pointer_width = "64")]
    for rela in relas {
        match reloc::r_type(rela.r_info) {
            // self::DTPMOD64 => {
            //     let reloc = (rela.r_offset as usize + bias) as *mut u64;
            //     unsafe {
            //         *reloc = 0; // just because
            //     }
            // },
            self::RELATIVE => {
                // get the actual symbols address given by adding the on-disk binary offset `r_offset` to the symbol with the actual address the linker was loaded at
                let reloc = (rela.r_offset as usize + bias) as *mut usize;
                // now set the content of this address to whatever is at the load bias + the addend
                // typically, this is all static, constant read only global data, like strings, constant ints, etc.
                unsafe {
                    // TODO: verify casting bias to an isize is correct
                    *reloc = (rela.r_addend + bias as i64) as usize;
                }
            },
            _ => ()
        }
    }
    if info.textrel {
        let res = utils::mmap::mprotect_phdrs(phdrs, bias, 0);
    }
}
