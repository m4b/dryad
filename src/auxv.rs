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

pub const AUX_CNT: usize = 38;

#[repr(C)]
pub struct Auxv {
    pub a_type: u64,
    pub a_val: u64
}

pub unsafe fn from_raw (auxv: *const Auxv) -> Vec<u64> {
    let mut aux: Vec<u64> = vec![0; AUX_CNT];
    let mut i: isize = 0;
    while (&*auxv.offset(i)).a_val != AT_NULL {
        let auxv_t = &*auxv.offset(i);
        // musl wants the aux a_val array to be indexed by AT_<TYPE>
        aux[auxv_t.a_type as usize] = auxv_t.a_val;
        i += 1;
    }
    aux
}

#[inline(always)]
fn str_of_idx (idx: u64) -> &'static str {
    match idx {
        AT_NULL => "AT_NULL",
        AT_IGNORE => "AT_IGNORE",
        AT_EXECFD => "AT_EXECFD",
        AT_PHDR => "AT_PHDR",
        AT_PHENT => "AT_PHENT",
        AT_PHNUM => "AT_PHNUM",
        AT_PAGESZ => "AT_PAGESZ",
        AT_BASE => "AT_BASE",
        AT_FLAGS => "AT_FLAGS",
        AT_ENTRY => "AT_ENTRY",
        AT_NOTELF => "AT_NOTELF",
        AT_UID => "AT_UID",
        AT_EUID => "AT_EUID",
        AT_GID => "AT_GID",
        AT_EGID => "AT_EGID",
        AT_PLATFORM => "AT_PLATFORM",
        AT_HWCAP => "AT_HWCAP",
        AT_CLKTCK => "AT_CLKTCK",
        AT_FPUCW => "AT_FPUCW",
        AT_DCACHEBSIZE => "AT_DCACHEBSIZE",
        AT_ICACHEBSIZE => "AT_ICACHEBSIZE",
        AT_UCACHEBSIZE => "AT_UCACHEBSIZE",
        AT_IGNOREPPC => "AT_IGNOREPPC",
        AT_SECURE => "AT_SECURE",
        AT_BASE_PLATFORM => "AT_BASE_PLATFORM",
        AT_RANDOM => "AT_RANDOM",
        AT_HWCAP2 => "AT_HWCAP2",
        AT_EXECFN => "AT_EXECFN",
        AT_SYSINFO => "AT_SYSINFO",
        AT_SYSINFO_EHDR => "AT_SYSINFO_EHDR",
        AT_L1I_CACHESHAPE => "AT_L1I_CACHESHAPE",
        AT_L1D_CACHESHAPE => "AT_L1D_CACHESHAPE",
        AT_L2_CACHESHAPE => "AT_L2_CACHESHAPE",
        AT_L3_CACHESHAPE => "AT_L3_CACHESHAPE",
        _ => "UNKNOWN_AT_TYPE",
    }
}

pub fn show (auxvs: &Vec<u64>) {
    for (i, auxv) in auxvs.iter().enumerate() {
        if *auxv != 0 {
            println!("{}: 0x{:x}", str_of_idx(i as u64), auxv)
        }
    }
}
