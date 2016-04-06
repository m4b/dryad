#![allow(non_camel_case_types)]

pub const AT_NULL: u64 = 0;
pub const AT_IGNORE: u64 = 1;
pub const AT_EXECFD: u64 = 2;
pub const AT_PHDR: u64 = 3;
pub const AT_PHENT: u64 = 4;
pub const AT_PHNUM: u64 = 5;
pub const AT_PAGESZ: u64 = 6;
pub const AT_BASE: u64 = 7;
pub const AT_FLAGS: u64 = 8;
pub const AT_ENTRY: u64 = 9;
pub const AT_NOTELF: u64 = 10;
pub const AT_UID: u64 = 11;
pub const AT_EUID: u64 = 12;
pub const AT_GID: u64 = 13;
pub const AT_EGID: u64 = 14;
pub const AT_PLATFORM: u64 = 15;
pub const AT_HWCAP: u64 = 16;
pub const AT_CLKTCK: u64 = 17;
pub const AT_FPUCW: u64 = 18;
pub const AT_DCACHEBSIZE: u64 = 19;
pub const AT_ICACHEBSIZE: u64 = 20;
pub const AT_UCACHEBSIZE: u64 = 21;
pub const AT_IGNOREPPC: u64 = 22;
pub const AT_SECURE: u64 = 23;
pub const AT_BASE_PLATFORM: u64 = 24;
pub const AT_RANDOM: u64 = 25;
pub const AT_HWCAP2: u64 = 26;
pub const AT_EXECFN: u64 = 31;
pub const AT_SYSINFO: u64 = 32;
pub const AT_SYSINFO_EHDR: u64 = 33;
pub const AT_L1I_CACHESHAPE: u64 = 34;
pub const AT_L1D_CACHESHAPE: u64 = 35;
pub const AT_L2_CACHESHAPE: u64 = 36;
pub const AT_L3_CACHESHAPE: u64 = 37;

#[repr(C)]
pub struct Elf64_auxv_t {
    pub a_type: u64,
    pub a_val: u64
}
