//#![feature(no_std, lang_items, asm, core, core_str_ext)]
//#![no_std]
#![feature(asm, libc)]
#![no_main]
//#![no_builtins]

#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_variables)]

//mod llvm_symbols;
mod auxv;
mod kernel_block;
mod utils;
mod image;
mod binary;
mod relocate;

use kernel_block::KernelBlock;
use utils::*;
use binary::elf::header;
use binary::elf::dyn;
use binary::elf::program_header;
//use binary::elf::rela;

extern crate libc;

//extern "C"
//void __attribute__((noinline)) __attribute__((visibility("default")))

// unused; someone figure out how to get gdb working when running as a dyld
extern "C" {
    fn rtld_db_dlactivity();
}

// elf abi requires _start; this must be in assembly because we need
// the raw stack pointer as the argument to _dryad_init
extern {
    fn _start();
    fn __init_tls(aux: *const u64); // pointer to aux vector indexed by AT_<TYPE> that musl likes
}

fn compute_load_bias(base:u64, phdrs:&[program_header::ProgramHeader]) -> u64 {
    for phdr in phdrs {
        if phdr.p_type == program_header::PT_LOAD {
            return base + (phdr.p_offset - phdr.p_vaddr);
        }
    }
    0
}

#[no_mangle]
pub extern fn _dryad_init(raw_args: *const u64) -> u64 {
    unsafe { write(&"dryad::_dryad_init\n"); }
    
    let block = KernelBlock::new(raw_args);
    unsafe { block.debug_print(); }

    let linker_image = image::Elf::new(&block);
    unsafe { linker_image.debug_print(); }

    unsafe {
        write(&"dryad::init_tls\n");
        __init_tls(block.get_aux().as_ptr());
    }

    // this is the linker's elf_header and program_header[0]
    unsafe {
        let elf_header = header::as_header(linker_image.base as *const u64);
        write(&"LINKER ELF\n");
        elf_header.debug_print();
        let addr = (linker_image.base + elf_header.e_phoff) as *const program_header::ProgramHeader;
        let linker_phdrs = program_header::to_phdr_array(addr, elf_header.e_phnum as usize);
        write(&"LINKER PHDRS\n");
        program_header::debug_print_phdrs(linker_phdrs);

        let load_bias = compute_load_bias(linker_image.base, &linker_phdrs);
        write(&"load bias: 0x");
        write_u64(load_bias, true);
        write(&"\n");
        if let Some(dynamic) = dyn::get_dynamic_array(load_bias, linker_phdrs) {
            write(&"LINKER _DYNAMIC\n");
            dyn::debug_print_dynamic(dynamic);
            let relocations = relocate::get_relocations(load_bias, dynamic);
            write(&"\nnumber of relocations: ");
            write_u64(relocations.len() as u64, false);
            write(&"\n");
            relocate::relocate(&relocations, load_bias);
            println!("{:?}", linker_phdrs);
        } else {
            write(&"<dryad> NO DYNAMIC for ");
            // TODO: add proper name value via slice
            write_chars_at(*block.argv, 0);
            write(&"\n");
        }
    }

    // TODO: refactor and remove, for testing
    // EXECUTABLE
    unsafe {
        let addr = linker_image.phdr as *const program_header::ProgramHeader;
        let phdrs = program_header::to_phdr_array(addr, linker_image.phnum as usize);
        println!("{:?}", phdrs);
        let mut base = 0;
        let mut load_bias = 0;
        for phdr in phdrs {
            if phdr.p_type == program_header::PT_PHDR {
                load_bias = linker_image.phdr - phdr.p_vaddr;
                base = linker_image.phdr - phdr.p_offset;
            }
        }
        write(&"load bias: 0x");
        write_u64(load_bias, true);
        write(&" base: 0x");
        write_u64(base, true);
        write(&"\n");

        if let Some(dynamic) = dyn::get_dynamic_array(load_bias, phdrs) {
            write(&"EXE _DYNAMIC\n");
            dyn::debug_print_dynamic(dynamic);
            let strtab = dyn::get_strtab(load_bias, dynamic);
            let needed = dyn::get_needed(dynamic, strtab, base, load_bias);
            dyn::print_needed(needed);

        } else {
            write(&"<dryad> NO DYNAMIC for ");
            // TODO: add proper name value via slice
            write_chars_at(*block.argv, 0);
            write(&"\n");
        }
    }

    if _start as *const u64 as u64 == linker_image.entry {
        // because it's _tradition_
        // (https://fossies.org/dox/glibc-2.22/rtld_8c_source.html)
        // line 786:
        // > Ho ho.  We are not the program interpreter!  We are the program itself!
        unsafe { write(&"-=|dryad====-\n"); }
        _exit(0);
        0
    } else {
        // commenting _exit will successfully
        // tranfer control (in my single test case ;))
        // to the program entry in test/test,
        // but segfaults when printf is called (obviously)
        // since we've done no dynamic linking
        _exit(0);
        linker_image.entry
    }
}
