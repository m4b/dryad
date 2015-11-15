//#![feature(no_std, lang_items, asm, core, core_str_ext)]
//#![no_std]
//#![no_builtins]
#![feature(asm, libc)]
#![no_main]

#![allow(dead_code)]
#![allow(unused_variables)]

//mod llvm_symbols;
mod auxv;
mod kernel_block;
mod utils;
mod image;
mod binary;
mod relocate;
mod link_map;
mod linker;

use kernel_block::KernelBlock;
use utils::*;
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

#[no_mangle]
pub extern fn _dryad_init(raw_args: *const u64) -> u64 {

    let block = KernelBlock::new(raw_args);
    unsafe { block.unsafe_print(); }

    let linker_base = block.getauxval(auxv::AT_BASE).unwrap();
    let entry  = block.getauxval(auxv::AT_ENTRY).unwrap();

    let start_addr = _start as *const u64 as u64;

    // without this,
    // following comparison fails for some inexplicable reason... yay for side-effectful printing again
    unsafe {
        write(&"start: 0x");
        write_u64(start_addr, true);
        write(&" entry: 0x");
        write_u64(entry, true);
        write(&"\n");
    }
    
    if start_addr == entry {
        // because it's _tradition_
        // (https://fossies.org/dox/glibc-2.22/rtld_8c_source.html)
        // line 786:
        // > Ho ho.  We are not the program interpreter!  We are the program itself!
        unsafe { write(&"-=|dryad====-\n"); }
        _exit(0);
        return 0;
    }
    
    match linker::Linker::new(linker_base) {
        Ok (dryad) => {
            // dryad has successfully relocated itself; time to init tls
            unsafe { __init_tls(block.get_aux().as_ptr()); }
            // EXECUTABLE
            println!("BEGIN EXE LINKING");
            // TODO:
            // * image::elf::new(<stuff>)
            // * dryad::link(image)
            unsafe {
                let phdr_addr = block.getauxval(auxv::AT_PHDR).unwrap();
                let phent  = block.getauxval(auxv::AT_PHENT).unwrap();
                let phnum  = block.getauxval(auxv::AT_PHNUM).unwrap();

                let addr = phdr_addr as *const program_header::ProgramHeader;
                let phdrs = program_header::to_phdr_array(addr, phnum as usize);
                println!("Program Headers: {:#?}", &phdrs);
                let mut base = 0;
                let mut load_bias = 0;
                for phdr in phdrs {
                    if phdr.p_type == program_header::PT_PHDR {
                        load_bias = phdr_addr - phdr.p_vaddr;
                        base = phdr_addr - phdr.p_offset;
                        break;
                    }
                }
                println!("load bias: {:x} base: {:x}", load_bias, base);

                if let Some(dynamic) = dyn::get_dynamic_array(load_bias, phdrs) {
                    println!("_DYNAMIC: {:#?}", dynamic);
                    let strtab = dyn::get_strtab(load_bias, dynamic);
                    let needed = dyn::get_needed(dynamic, strtab, base, load_bias);
                    println!("Needed: {:#?}", needed);
                } else {
                    //            println!("<dryad> NO DYNAMIC for {}", *block.argv);
                }
            }
            
            // commenting _exit will successfully
            // tranfer control (in my single test case ;))
            // to the program entry in test/test,
            // but segfaults when printf is called (obviously)
            // since we've done no dynamic linking
            _exit(0);
            entry
        },
        Err (msg) => {
            unsafe { write(&msg); }
            _exit(1);
            0
        }
    }
}
