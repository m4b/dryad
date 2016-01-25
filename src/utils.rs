// leave this to allow easy breakpoints on assembly wrappers like _write for now
#![allow(private_no_mangle_fns)]

use std::str;
use std::slice;

// TODO: make this a mod like asm::

#[no_mangle]
pub extern fn _exit(code: u64) {
    unsafe {
        asm!("movq $$60, %rax
              syscall"
             :
             : "{rdi}"(code)
             );
    }
}

// this comes from asm.s
extern {
    fn _print(msg: *const u8, len: u64);
}

/*
fn _print(msg: *const u8, len: u64) {
    unsafe {
        let slice = slice::from_raw_parts(msg, len as usize);
        println!("{:?}", &slice);
    }
}
*/

#[no_mangle]
pub unsafe extern fn write(msg: &str){
    _print(msg.as_ptr(), msg.len() as u64);
}

/*
#[cfg(debug_assertions)]
macro_rules! debug_write {
    ($($t:tt)*) => {
        unsafe {
            write($($t)*)
        }
    }
}

#[cfg(not(debug_assertions))]
macro_rules! debug_write {
    ($($t:tt)*) => {
    }
}
*/

/*
// this is _totally_ broken and is massively side-effectful and unpredicatable
#[no_mangle]
pub extern fn write(msg: &str) {
    unsafe {
        asm!("pushq %rdi
              pushq %rax
              pushq %rdx
              pushq %rsi
              movq $$1, %rax
              movq $$1, %rdi
              syscall
              popq %rsi
              popq %rdx
              popq %rax
              popq %rdi
              "
             :
             : "{rsi}"(msg.as_ptr()), "{rdx}"(msg.len())
//             : "{rdi}","{rax}", "{rdx}", "{rsi}"
             //             :"{rsi}"(msg.as_ptr()), "{rdx}"(msg.len())
             //
             );
    }
}
*/
fn digit_to_char_code(i: u8) -> u8 {
    if i <= 9 {
        i + 48
    }else{
        0
    }
}

fn num_digits(i: u64) -> usize {
    if i == 0 {
        1
    } else {
        let mut count = 0;
        let mut current = i;
        while current > 0 {
            current /= 10;
            count += 1;
        }
        count
    } 
}

#[test]
fn num_digits_t() {
    assert_eq!(num_digits(0), 1);
    assert_eq!(num_digits(10), 2);
    assert_eq!(num_digits(99), 2);
    assert_eq!(num_digits(999), 3);
}

#[no_mangle]
pub unsafe extern fn write_u64(i: u64, base16: bool) {
    if base16 {
        write(to_hex(&i, &mut [0; 16]));
    } else {
        let count = num_digits(i);
        let mut _stack = [0; 20];
        let mut chars = &mut _stack[0..count];
        let mut place = count;
        let mut current = i;
        let mut digit;
        loop {
            digit = current % 10;
            current = (current - digit) / 10;
            place -= 1;
            chars[place] = digit_to_char_code(digit as u8);
            if current <= 0 { break; }
        }
        write(str::from_utf8(chars).unwrap());
    }
}

fn to_hex<'a>(i: &u64, output: &'a mut[u8; 16]) -> &'a str {
    let mut input = *i;
    let hex = b"0123456789abcdef";
    let mut buffer = [0; 16];
    let mut i = 0;
    let mut j = 0;
    if input == 0 {
        buffer[0] = hex[0];
        i = 1;
    } else {
        while input > 0 {
            buffer[i] = hex[(input % 16) as usize];
            input = input / 16;
            i += 1;
        }
    }

    while i > 0 {
        i -= 1;
        output[j] = buffer[i];
        j += 1;
    }

    str::from_utf8(output).unwrap().trim_matches('\0')
}

pub fn as_str<'a>(cs: *const u8) -> &'a str {
    if cs.is_null() {
        ""
    }else {
        unsafe {
            let mut i = 0;
            let mut c = *cs;
            while c != 0 {
                i += 1;
                c = *cs.offset(i);
            }
            let slice = slice::from_raw_parts(cs, i as usize);
            str::from_utf8(slice).unwrap()
        }
    }
}

pub fn str_at<'a>(cs: *const u8, offset: isize) -> &'a str {
    if cs.is_null() {
        ""
    }else {
        let mut i = 0;
        unsafe {
            let ptr = cs.offset(offset);
            let mut c = *ptr;
            while c != 0 {
                i += 1;
                c = *ptr.offset(i);
            }
            let slice = slice::from_raw_parts(ptr, i as usize);
            str::from_utf8(slice).unwrap()
        }
    }
}


pub unsafe extern fn write_chars_at(cs: *const u8) {
    write(as_str(cs));
}

pub mod page {
   // from <sys/user.h>
    pub const PAGE_SHIFT:u64 = 12;
    pub const PAGE_SIZE:u64 = 1 << PAGE_SHIFT;
    pub const PAGE_MASK:u64 = !(PAGE_SIZE - 1);

    // from bionic
    /// Returns the address of the page containing address 'x'.
    #[inline(always)]
    pub fn page_start (x:u64) -> u64 { x & PAGE_MASK }

    /// Returns the offset of address 'x' in its page.
    #[inline(always)]
    pub fn page_offset (x:u64) -> u64 { x & !PAGE_MASK }

    /// Returns the address of the next page after address 'x', unless 'x' is
    /// itself at the start of a page.
    #[inline(always)]
    pub fn page_end (x:u64) -> u64 { page_start(x + (PAGE_SIZE - 1)) }

}

pub mod mmap {

    pub const PROT_READ:isize = 0x1; /* Page can be read.  */
    pub const PROT_WRITE:isize = 0x2; /* Page can be written.  */
    pub const PROT_EXEC:isize = 0x4; /* Page can be executed.  */
    pub const PROT_NONE:isize = 0x0; /* Page can not be accessed.  */
    pub const PROT_GROWSDOWN:isize = 0x01000000; /* Extend change to start of growsdown vma (mprotect only).  */
    pub const PROT_GROWSUP:isize = 0x02000000; /* Extend change to start of growsup vma (mprotect only).  */

    /* Sharing types (must choose one and only one of these).  */
    pub const MAP_SHARED:isize = 0x01; /* Share changes.  */
    pub const MAP_PRIVATE:isize = 0x02; /* Changes are private.  */
    pub const MAP_ANONYMOUS:isize = 0x20; // just guessing, this is wrapped in a ifdef with __MAP_ANONYMOUS as the value
    /* Other flags.  */
    pub const MAP_FIXED:isize = 0x10; /* Interpret addr exactly.  */

    /// map failed, from sys/mman.h, technically ((void *) - 1) ...
    pub const MAP_FAILED:u64 = !0;

    // from musl libc
    extern {
        pub fn mmap(addr: *const u64, len: usize, prot: isize, flags: isize, fildes: isize, off: usize) -> u64;
    }

}
