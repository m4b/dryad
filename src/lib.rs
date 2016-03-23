//#![feature(no_std, lang_items, asm, core, core_str_ext)]
//#![no_std]
#![feature(asm, libc)]
#![no_main]

//#![feature(std_panic, recover)]

#![allow(dead_code)] // yells about consts otherwise
#![allow(unused_variables)]

/// Dryad --- the world's first non-functional, yet-to-be-implemented, might be impossible or more likely inefficient --- parallel, dynamic linker.
/// Many, many thanks to Mutabah, durka42, aatch, tilpner, niconii, bluss, steveklabnik and so many others on the IRC channel for answering my stupid questions.
/// TODO: add support for invoking ./dryad <binary> <args>

mod auxv;
mod kernel_block;
mod utils;
mod binary;
pub mod runtime;
pub mod linker;

use kernel_block::KernelBlock;
use linker::Linker;
use utils::*;

extern crate libc;

// below is gcc attrs for this function...
//extern "C"
//void __attribute__((noinline)) __attribute__((visibility("default")))
// unused; someone figure out how to get gdb working when running as a dyld
extern {
    fn rtld_db_dlactivity();
}

extern {
    /// ELF abi requires `_start`; this must be in assembly because we need
    /// the raw stack pointer as the argument to `_dryad_init`;
    /// i.e., kernel calls symbol `_start` on dynamic linker with the kernel argument block, etc.,
    /// which in our case then calls _back_ into `dryad_init` with the pointer to the raw arguments that form the kernel block
    /// see `arch/x86/asm.s`
    fn _start();
}

#[no_mangle]
pub extern fn dryad_init (raw_args: *const u64) -> u64 {

    // the linker is currently tied to the lifetime of the kernel block... but really it's static
    let block = KernelBlock::new(raw_args);
    let linker_base = block.getauxval(auxv::AT_BASE).unwrap();
    let entry  = block.getauxval(auxv::AT_ENTRY).unwrap();

    let start_addr = _start as *const u64 as u64;    
    // TODO: add support for invoking ./dryad <binary> <args>
    if start_addr == entry {
        // because it's _tradition_
        // (https://fossies.org/dox/glibc-2.22/rtld_8c_source.html)
        // line 786:
        // > Ho ho.  We are not the program interpreter!  We are the program itself!
        println!("-=|dryad====-\nHo ho.  We are not the program interpreter!  We are the program itself!"); // TODO: add box drawing random character gen here cause it'll be _cute_
        _exit(0);
        return 0xd47ad // to make compiler happy
    }

    match Linker::new(linker_base, &block) {
        Ok (dryad) => {
            if let Err(msg) = dryad.link(&block) {
                println!("{}", msg);
                _exit(1);
                0xd47ad
            } else {
                entry
            }
        },
        Err (msg) => {
            // relocating self failed somehow; we try to write the error message and exit
            unsafe { write(&msg); }
            _exit(1);
            0xd47ad
        }
    }
}
