//#![feature(no_std, lang_items, asm, core, core_str_ext)]
//#![no_std]
#![feature(asm)]
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

use kernel_block::KernelBlock;
use utils::*;
use binary::elf::header;
use binary::elf::dyn;
use binary::elf::program_header;

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
}

#[no_mangle]
pub extern fn _dryad_init(raw_args: *const u64) -> u64 {
    unsafe { write(&"dryad::_dryad_init\n"); }
    
    let block = KernelBlock::new(raw_args);
    unsafe { block.debug_print(); }

    let linker_image = image::Elf::new(&block);
    unsafe { linker_image.debug_print(); }

    unsafe {
        // this is the linker's elf_header and program_header[0]
        let elf_header = header::as_header(linker_image.base as *const u64);
//        println!("{:?}", elf_header);
        write(&"LINKER ELF\n");
        elf_header.debug_print();
        let addr = (linker_image.base + elf_header.e_phoff) as *const program_header::ProgramHeader;
        let linker_phdrs = program_header::to_phdr_array(addr, elf_header.e_phnum as usize);
        write(&"LINKER PHDRS\n");
        program_header::debug_print_phdrs(linker_phdrs);
    }

    // TODO: refactor and remove, for testing
    unsafe {
        let addr = linker_image.phdr as *const program_header::ProgramHeader;

        let phdrs = program_header::to_phdr_array(addr, linker_image.phnum as usize);
        /*
        write(&"num phdrs: ");
        write_u64(phdrs.len() as u64, false);
        write(&"\n");
        */
        if let Some(dynamic) = dyn::get_dynamic_array(phdrs) {
            let strtab = dyn::get_strtab(dynamic);
            let base = linker_image.base;
            let needed = dyn::get_needed(dynamic, strtab, base);
//            write(&"EXE _DYNAMIC\n");
//            dyn::debug_print_dynamic(dynamic);

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
