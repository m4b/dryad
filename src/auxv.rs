//#![no_main]
//#![feature(no_std)]
//#![no_std]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub const AT_NULL:u64 = 0;
pub const AT_IGNORE:u64 = 1;
pub const AT_EXECFD:u64 = 2;
pub const AT_PHDR:u64 = 3;
pub const AT_PHENT:u64 = 4;
pub const AT_PHNUM:u64 = 5;
pub const AT_PAGESZ:u64 = 6;
pub const AT_BASE:u64 = 7;
pub const AT_FLAGS:u64 = 8;
pub const AT_ENTRY:u64 = 9;
pub const AT_NOTELF:u64 = 10;
pub const AT_UID:u64 = 11;
pub const AT_EUID:u64 = 12;
pub const AT_GID:u64 = 13;
pub const AT_EGID:u64 = 14;
pub const AT_PLATFORM:u64 = 15;
pub const AT_HWCAP:u64 = 16;
pub const AT_CLKTCK:u64 = 17;
pub const AT_FPUCW:u64 = 18;
pub const AT_DCACHEBSIZE:u64 = 19;
pub const AT_ICACHEBSIZE:u64 = 20;
pub const AT_UCACHEBSIZE:u64 = 21;
pub const AT_IGNOREPPC:u64 = 22;
pub const AT_SECURE:u64 = 23;
pub const AT_BASE_PLATFORM:u64 = 24;
pub const AT_RANDOM:u64 = 25;
pub const AT_HWCAP2:u64 = 26;
pub const AT_EXECFN:u64 = 31;
pub const AT_SYSINFO:u64 = 32;
pub const AT_SYSINFO_EHDR:u64 = 33;
pub const AT_L1I_CACHESHAPE:u64 = 34;
pub const AT_L1D_CACHESHAPE:u64 = 35;
pub const AT_L2_CACHESHAPE:u64 = 36;
pub const AT_L3_CACHESHAPE:u64 = 37;

#[derive(PartialEq)]
#[repr(C)]
pub enum AT {
    NULL,
    IGNORE,
    EXECFD,
    PHDR,
    PHENT,
    PHNUM,
    PAGESZ,
    BASE,
    FLAGS,
    ENTRY,
    NOTELF,
    UID,
    EUID,
    GID,
    EGID,
    CLKTCK,
    PLATFORM,
    HWCAP,
    FPUCW,
    DCACHEBSIZE,
    ICACHEBSIZE,
    UCACHEBSIZE,
    IGNOREPPC,
    SECURE,
    BASE_PLATFORM,
    RANDOM,
    HWCAP2,
    EXECFN,
    SYSINFO,
    SYSINFO_EHDR,
    L1I_CACHESHAPE,
    L1D_CACHESHAPE,
    L2_CACHESHAPE,
    L3_CACHESHAPE,
    UNKNOWN (u64),
}

pub fn u64_to_at (t: u64) -> AT {
    match t {
        0 => AT::NULL,
        1 => AT::IGNORE,
        2 => AT::EXECFD,
        3 => AT::PHDR,
        4 => AT::PHENT,
        5 => AT::PHNUM,
        6 => AT::PAGESZ,
        7 => AT::BASE,
        8 => AT::FLAGS,
        9 => AT::ENTRY,
        10 => AT::NOTELF,
        11 => AT::UID,
        12 => AT::EUID,
        13 => AT::GID,
        14 => AT::EGID,
        15 => AT::PLATFORM,
        16 => AT::HWCAP,
        17 => AT::CLKTCK,
        18 => AT::FPUCW,
        19 => AT::DCACHEBSIZE,
        20 => AT::ICACHEBSIZE,
        21 => AT::UCACHEBSIZE,
        22 => AT::IGNOREPPC,
        23 => AT::SECURE,
        24 => AT::BASE_PLATFORM,
        25 => AT::RANDOM,
        26 => AT::HWCAP2,
        31 => AT::EXECFN,
        32 => AT::SYSINFO,
        33 => AT::SYSINFO_EHDR,
        34 => AT::L1I_CACHESHAPE,
        35 => AT::L1D_CACHESHAPE,
        36 => AT::L2_CACHESHAPE,
        37 => AT::L3_CACHESHAPE,
        _  => AT::UNKNOWN(t),
    }
}

#[repr(C)]
pub struct Elf64_auxv_t {
    pub a_type: u64, // AT as type doesn't work, kernel_block::getauxv returns a_type instead of a_val somehow; c repr doesn't help
    pub a_val: u64
}

/*
        NULL		0
            IGNORE	1
            EXECFD	2
            PHDR		3
            PHENT	4
            PHNUM	5
            PAGESZ	6
            BASE		7
            FLAGS	8
            ENTRY	9
            NOTELF	10
            UID		11
            EUID		12
            GID		13
            EGID		14
            CLKTCK	17
            PLATFORM	15
            HWCAP	16
            FPUCW	18
            DCACHEBSIZE	19
            ICACHEBSIZE	20
            UCACHEBSIZE	21
            IGNOREPPC	22
            SECURE	23
            BASE_PLATFORM 24
            RANDOM	25
            HWCAP2	26
            EXECFN	31
            SYSINFO	32
            SYSINFO_EHDR	33
            L1I_CACHESHAPE	34
            L1D_CACHESHAPE	35
            L2_CACHESHAPE	36
            L3_CACHESHAPE	37
 */
/*
NULL		0		/* End of vector */
IGNORE	1		/* Entry should be ignored */
EXECFD	2		/* File descriptor of program */
PHDR		3		/* Program headers for program */
PHENT	4		/* Size of program header entry */
PHNUM	5		/* Number of program headers */
PAGESZ	6		/* System page size */
BASE		7		/* Base address of interpreter */
FLAGS	8		/* Flags */
ENTRY	9		/* Entry point of program */
NOTELF	10		/* Program is not ELF */
UID		11		/* Real uid */
EUID		12		/* Effective uid */
GID		13		/* Real gid */
EGID		14		/* Effective gid */
CLKTCK	17		/* Frequency of times() */

/* Some more special a_type values describing the hardware.  */
PLATFORM	15		/* String identifying platform.  */
HWCAP	16		/* Machine-dependent hints about
					   processor capabilities.  */

/* This entry gives some information about the FPU initialization
   performed by the kernel.  */
FPUCW	18		/* Used FPU control word.  */

/* Cache block sizes.  */
DCACHEBSIZE	19		/* Data cache block size.  */
ICACHEBSIZE	20		/* Instruction cache block size.  */
UCACHEBSIZE	21		/* Unified cache block size.  */

/* A special ignored value for PPC, used by the kernel to control the
   interpretation of the AUXV. Must be > 16.  */
IGNOREPPC	22		/* Entry should be ignored.  */

#define	AT_SECURE	23		/* Boolean, was exec setuid-like?  */

BASE_PLATFORM 24		/* String identifying real platforms.*/

RANDOM	25		/* Address of 16 random bytes.  */

HWCAP2	26		/* More machine-dependent hints about
					   processor capabilities.  */

EXECFN	31		/* Filename of executable.  */

/* Pointer to the global system page used for system calls and other
   nice things.  */
SYSINFO	32
SYSINFO_EHDR	33

/* Shapes of the caches.  Bits 0-3 contains associativity; bits 4-7 contains
   log2 of line size; mask those to get cache size.  */
L1I_CACHESHAPE	34
L1D_CACHESHAPE	35
L2_CACHESHAPE	36
L3_CACHESHAPE	37
}

*/

