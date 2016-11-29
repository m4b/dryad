use std::str;
use std::slice;

#[cfg(feature = "color")]
macro_rules! colour {
    ($c:ident: $str:expr) =>
    (colorify!($c: $str))
}

#[cfg(not(feature = "color"))]
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


pub fn _exit(code: u64) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!("movq $$60, %rax
              syscall"
             :
             : "{rdi}"(code)
             );
    }
}

fn asm_write(msg: *const u8, len: u64){
    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!("\
              mov %rsi, %rdx
              mov %rdi, %rsi
              movq $$1, %rax
              movq $$1, %rdi
              syscall
              "
             :
             :"{rdi}"(msg), "{rsi}"(len)
             : "rdi","rax", "rdx", "rsi"
             : "alignstack", "volatile"
             );
    }
}

pub fn write(msg: &str){
    asm_write(msg.as_ptr(), msg.len() as u64);
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

pub fn write_u64(i: u64, base16: bool) {
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
    pub const PAGE_SHIFT: usize    = 12;
    pub const PAGE_SIZE: usize     = 1 << PAGE_SHIFT;
    const PAGE_SIZE_MINUS_1: usize = PAGE_SIZE - 1;
    pub const PAGE_MASK: usize     = !PAGE_SIZE_MINUS_1;

    // from bionic
    /// Returns the address of the page containing address 'x'.
    #[inline(always)]
    pub fn page_start (x: usize) -> usize { x & PAGE_MASK }

    /// Returns the offset of address 'x' in its page.
    #[inline(always)]
    pub fn page_offset (x: usize) -> usize { x & PAGE_SIZE_MINUS_1 }

    /// Returns the address of the next page after address 'x', unless 'x' is
    /// itself at the start of a page.
    #[inline(always)]
    pub fn page_end (x: usize) -> usize { page_start(x + PAGE_SIZE_MINUS_1) }

    #[test]
    fn t_page_start () {
        assert_eq!(page_start(0x1000), 0x1000)
    }

}

pub mod mmap {
    use libc;
    use utils::page;
    use std::fs::File;
    use std::os::unix::io::AsRawFd;
    use elf::program_header;

    // from /usr/include/bits/mman.h and mman-linux.h
    // I'm even warned not to "include" this, so I will anyway: # error "Never use <bits/mman-linux.h> directly; include <sys/mman.h> instead."
    pub const PROT_READ: isize = 0x1; // Page can be read
    pub const PROT_WRITE: isize = 0x2; // Page can be written
    pub const PROT_EXEC: isize = 0x4; // Page can be executed
    pub const PROT_NONE: isize = 0x0; // Page can not be accessed
    pub const PROT_GROWSDOWN: isize = 0x01000000; // Extend change to start of growsdown vma (mprotect only)
    pub const PROT_GROWSUP: isize = 0x02000000; // Extend change to start of growsup vma (mprotect only)

    // Sharing types (must choose one and only one of these)
    pub const MAP_FILE: isize = 0x0; // no flag bits to map a file
    pub const MAP_SHARED: isize = 0x01; // Share changes
    pub const MAP_PRIVATE: isize = 0x02; // Changes are private
    pub const MAP_ANONYMOUS: isize = 0x20; // just guessing, this is wrapped in a ifdef with __MAP_ANONYMOUS as the value
    // Other flags
    pub const MAP_DENYWRITE: isize = 0x800;
    pub const MAP_COPY: isize = MAP_PRIVATE | MAP_DENYWRITE;
    pub const MAP_FIXED: isize = 0x10; // Interpret addr exactly

    /// map failed, from sys/mman.h, technically ((void *) - 1) ...
    pub const MAP_FAILED: usize = !0;

    // from musl libc
    extern {
        fn mmap64(addr: *const usize, len: usize, prot: isize, flags: libc::c_int, fildes: libc::c_int, off: usize) -> usize;
        fn mprotect(addr: *const libc::c_void, len: libc::size_t, prot: libc::c_int) -> libc::c_int;
    }

    #[inline(always)]
    pub unsafe fn mmap(addr: *const usize, len: usize, prot: isize, flags: libc::c_int, fildes: libc::c_int, off: usize) -> usize {
        mmap64(addr, len, prot, flags, fildes, off)
    }

    #[inline(always)]
    pub fn pflags_to_prot (x: u32) -> isize {
        use elf::program_header::{PF_X, PF_R, PF_W};

        // I'm a dick for writing this/copying maniac C programmer implementations: but it checks the flag to see if it's the PF value,
        // and returns the appropriate mmap version, and logical ORs this for use in the mmap prot argument
        (if x & PF_X == PF_X { PROT_EXEC } else { 0 }) |
        (if x & PF_R == PF_R { PROT_READ } else { 0 }) |
        (if x & PF_W == PF_W { PROT_WRITE } else { 0 })
    }

    pub fn mprotect_phdrs (phdrs: &[program_header::ProgramHeader], bias: usize, flags: isize) -> bool {
        for phdr in phdrs {
            if phdr.p_type == program_header::PT_LOAD {
                let seg_page_start = page::page_start(phdr.p_vaddr as usize) + bias;
                let seg_page_end = page::page_end((phdr.p_vaddr + phdr.p_memsz) as usize) + bias;
                let mut prot = pflags_to_prot(phdr.p_flags);
                if (flags as isize & PROT_WRITE) != 0 {
                    // bionic says: make sure we're never simultaneously writable / executable
                    prot &= !PROT_EXEC;
                }
                let new_flags = prot | flags;
                // avoid the syscall unless we're restoring the program header flags (flags == 0)
                if flags != 0 && new_flags == prot {
                    return true;
                }
                let ret = unsafe { mprotect(seg_page_start as *const libc::c_void, (seg_page_end - seg_page_start) as libc::size_t, new_flags as libc::c_int) };
                if ret < 0 { return false }
            }
        }
        true
    }

    #[inline(always)]
    fn map_fragment(fd: &File, base: usize, offset: usize, size: usize) -> Result<(usize, usize, *const usize), String> {
        use utils::page;
        let offset = base + offset;
        let page_min = page::page_start(offset);
        let end_offset = offset + size as usize;
        let end_offset = end_offset + page::page_offset(offset);

        let map_size: usize = (end_offset - page_min) as usize;

        if map_size < size {
            return Err (format!("Error: file {:#?} has map_size = {} < size = {}, aborting", fd, map_size, size))
        }

        let map_start = unsafe { mmap(0 as *const usize,
                                            map_size,
                                            PROT_READ,
                                            MAP_PRIVATE as libc::c_int,
                                            fd.as_raw_fd() as libc::c_int,
                                            page_min as usize) };

        if map_start == MAP_FAILED {

            Err (format!("Error: mmap failed for {:#?} with errno {}, aborting", fd, super::get_errno()))

        } else {

            let data = (map_start + page::page_offset(offset)) as *const usize;
            Ok ((map_start, map_size, data))
        }
    }

}
