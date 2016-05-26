// leave this to allow easy breakpoints on assembly wrappers like _write for now
#![allow(private_no_mangle_fns)]

use std::str;
use std::slice;

#[cfg(not(feature = "no_color"))]
macro_rules! colour {
    ($c:ident: $str:expr) =>
    (colorify!($c: $str))
}

#[cfg(feature = "no_color")]
macro_rules! colour {
    ($c:ident: $str:expr) =>
        ($str)
}

macro_rules! bracket {
    ($c:ident: $str:expr) =>
       ( concat!(colour!(white_bold2: "<"), colour!($c: $str), colour!(white_bold2: ">")) )
}

macro_rules! dbg {
    ($dbg:expr, $fmt:expr) =>
        (if $dbg {
        ( print!(
            concat!(bracket!(green: "dryad"), " ", $fmt, "\n")))
        });
    ($dbg:expr, $fmt:expr, $($arg:tt)*) =>
        (if $dbg {
        ( print!(
            concat!(bracket!(green: "dryad"), " ", $fmt, "\n"),
            $($arg)*) );
        });
}

macro_rules! dbgc {
    ($c:ident: $dbg:expr, $prefix:expr, $fmt:expr) =>
        ( if $dbg {
            ( print!(
                concat!(bracket!($c: $prefix), " ", $fmt, "\n")) )
        });

    ($c:ident: $dbg:expr, $prefix:expr, $fmt:expr, $($arg:tt)*) =>
        ( if $dbg {
            ( print!(
                concat!(bracket!($c: $prefix), " ", $fmt, "\n"),
                $($arg)*) );
        } );
}


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
    pub fn _print(msg: *const u8, len: u64);
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

extern {
    /// libc #defines errno *(__errno_location()) ... so errno isn't a symbol in the actual binary and accesses will segfault us. yay.
    fn __errno_location() -> *const i32;
}

#[inline(always)]
pub fn get_errno () -> i32 {
    unsafe { *__errno_location() }
}

/// We're required to set the rust panic hook/handler because otherwise we segfault in `static_dl_iterate_phdr` when libstd tries to unwind the stack
/// **NB**: Make sure this is called _after_ relocation, since we need to allocate the closure on the heap
pub fn set_panic () {
    ::std::panic::set_hook(Box::new(|panic| {
        dbgc!(orange_bold: true, "dryad.panic", r#"Thamus, are you there? When you reach Palodes, take care to proclaim that the great god Pan is dead."#);
        if let Some(location) = panic.location() {
            println!("-=|dryad====- died in {}:{}", location.file(), location.line());
        }
        let payload = panic.payload ();
        // pretty much copied from libstd's panicking default, cause it's pretty
        let msg = match payload.downcast_ref::<&'static str>() {
            Some(s) => s,
            None => match payload.downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            }
        };
        dbgc!(orange_bold: true, "dryad.panic", "Died because {}", msg);
        _exit(1);
    }));
}

pub mod page {
   // from <sys/user.h>
    pub const PAGE_SHIFT: u64    = 12;
    pub const PAGE_SIZE: u64     = 1 << PAGE_SHIFT;
    const PAGE_SIZE_MINUS_1: u64 = PAGE_SIZE - 1;
    pub const PAGE_MASK: u64     = !PAGE_SIZE_MINUS_1;

    // from bionic
    /// Returns the address of the page containing address 'x'.
    #[inline(always)]
    pub fn page_start (x: u64) -> u64 { x & PAGE_MASK }

    /// Returns the offset of address 'x' in its page.
    #[inline(always)]
    pub fn page_offset (x: u64) -> u64 { x & PAGE_SIZE_MINUS_1 }

    /// Returns the address of the next page after address 'x', unless 'x' is
    /// itself at the start of a page.
    #[inline(always)]
    pub fn page_end (x: u64) -> u64 { page_start(x + PAGE_SIZE_MINUS_1) }

    #[test]
    fn t_page_start () {
        assert_eq!(page_start(0x1000), 0x1000)
    }

}

pub mod mmap {
    use std::os::raw::{c_int};
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    pub const PROT_READ: isize = 0x1; /* Page can be read.  */
    pub const PROT_WRITE: isize = 0x2; /* Page can be written.  */
    pub const PROT_EXEC: isize = 0x4; /* Page can be executed.  */
    pub const PROT_NONE: isize = 0x0; /* Page can not be accessed.  */
    pub const PROT_GROWSDOWN: isize = 0x01000000; /* Extend change to start of growsdown vma (mprotect only).  */
    pub const PROT_GROWSUP: isize = 0x02000000; /* Extend change to start of growsup vma (mprotect only).  */

    /* Sharing types (must choose one and only one of these).  */
    pub const MAP_FILE: isize = 0x0; /* no flag bits to map a file  */
    pub const MAP_SHARED: isize = 0x01; /* Share changes.  */
    pub const MAP_PRIVATE: isize = 0x02; /* Changes are private.  */
    pub const MAP_ANONYMOUS: isize = 0x20; // just guessing, this is wrapped in a ifdef with __MAP_ANONYMOUS as the value
    /* Other flags.  */
    pub const MAP_DENYWRITE: isize = 0x800;
    pub const MAP_COPY: isize = MAP_PRIVATE | MAP_DENYWRITE;
    pub const MAP_FIXED: isize = 0x10; /* Interpret addr exactly.  */

    /// map failed, from sys/mman.h, technically ((void *) - 1) ...
    pub const MAP_FAILED: u64 = !0;

    // from musl libc
    extern {
        fn mmap64(addr: *const u64, len: usize, prot: isize, flags: c_int, fildes: c_int, off: usize) -> u64;
    }

    #[inline(always)]
    pub unsafe fn mmap(addr: *const u64, len: usize, prot: isize, flags: c_int, fildes: c_int, off: usize) -> u64 {
        mmap64(addr, len, prot, flags, fildes, off)
    }

    #[inline(always)]
    fn map_fragment(fd: &File, base: u64, offset: u64, size: usize) -> Result<(u64, usize, *const u64), String> {
        use utils::page;
        let offset = base + offset;
        let page_min = page::page_start(offset);
        let end_offset = offset + size as u64;
        let end_offset = end_offset + page::page_offset(offset);

        let map_size: usize = (end_offset - page_min) as usize;

        if map_size < size {
            return Err (format!("<dryad> Error: file {:#?} has map_size = {} < size = {}, aborting", fd, map_size, size))
        }

        let map_start = unsafe { mmap(0 as *const u64,
                                            map_size,
                                            PROT_READ,
                                            MAP_PRIVATE as c_int,
                                            fd.as_raw_fd() as c_int,
                                            page_min as usize) };

        if map_start == MAP_FAILED {

            Err (format!("<dryad> Error: mmap failed for {:#?} with errno {}, aborting", fd, super::get_errno()))

        } else {

            let data = (map_start + page::page_offset(offset)) as *const u64;
            Ok ((map_start, map_size, data))
        }
    }

}
