//#![feature(no_std, lang_items, asm, core, core_str_ext)]
//#![no_std]
#![feature(asm, libc)]
#![feature(convert)]
#![no_main]

#![allow(dead_code)] // yells about consts otherwise
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

extern crate libc;

// below is gcc attrs for this function...
//extern "C"
//void __attribute__((noinline)) __attribute__((visibility("default")))
// unused; someone figure out how to get gdb working when running as a dyld
extern "C" {
    fn rtld_db_dlactivity();
}

extern {
    /// elf abi requires `_start`; this must be in assembly because we need
    /// the raw stack pointer as the argument to `_dryad_init`;
    /// i.e., kernel calls symbol `_start` on dynamic linker with the kernel argument block, etc.,
    /// which in our case then calls _back_ into `dryad_init`
    fn _start();
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
        return 0xd47ad // to make compiler happy
    }
    
    match linker::Linker::new(linker_base, &block) {
        Ok (dryad) => {
            println!("Dryad:\n  {:#?}", &dryad);
            println!("BEGIN EXE LINKING");
            let name = utils::as_str(block.argv[0]);
            let phdr_addr = block.getauxval(auxv::AT_PHDR).unwrap();
            let phnum  = block.getauxval(auxv::AT_PHNUM).unwrap();
            let main_image = image::elf::ElfExec::new(name, phdr_addr, phnum as usize);
            println!("Main Image:\n  {:#?}", &main_image);

            println!("<dryad> Final result: {:?}", dryad.link(main_image));
            
            // commenting _exit will successfully
            // tranfer control (in my single test case ;))
            // to the program entry in test/test,
            // but segfaults when printf is called (obviously)
            // since we've done no dynamic linking
            _exit(0);
            entry
        },
        Err (msg) => {
            // relocating self failed somehow; we write the error message and exit
            unsafe { write(&msg); }
            _exit(1);
            0xd47ad
        }
    }
}
