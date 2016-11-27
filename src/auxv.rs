pub const AT_NULL: usize = 0;
pub const AT_IGNORE: usize = 1;
pub const AT_EXECFD: usize = 2;
pub const AT_PHDR: usize = 3;
pub const AT_PHENT: usize = 4;
pub const AT_PHNUM: usize = 5;
pub const AT_PAGESZ: usize = 6;
pub const AT_BASE: usize = 7;
pub const AT_FLAGS: usize = 8;
pub const AT_ENTRY: usize = 9;
pub const AT_NOTELF: usize = 10;
pub const AT_UID: usize = 11;
pub const AT_EUID: usize = 12;
pub const AT_GID: usize = 13;
pub const AT_EGID: usize = 14;
pub const AT_PLATFORM: usize = 15;
pub const AT_HWCAP: usize = 16;
pub const AT_CLKTCK: usize = 17;
pub const AT_FPUCW: usize = 18;
pub const AT_DCACHEBSIZE: usize = 19;
pub const AT_ICACHEBSIZE: usize = 20;
pub const AT_UCACHEBSIZE: usize = 21;
pub const AT_IGNOREPPC: usize = 22;
pub const AT_SECURE: usize = 23;
pub const AT_BASE_PLATFORM: usize = 24;
pub const AT_RANDOM: usize = 25;
pub const AT_HWCAP2: usize = 26;
pub const AT_EXECFN: usize = 31;
pub const AT_SYSINFO: usize = 32;
pub const AT_SYSINFO_EHDR: usize = 33;
pub const AT_L1I_CACHESHAPE: usize = 34;
pub const AT_L1D_CACHESHAPE: usize = 35;
pub const AT_L2_CACHESHAPE: usize = 36;
pub const AT_L3_CACHESHAPE: usize = 37;

pub const AUX_CNT: usize = 38;

#[repr(C)]
pub struct Auxv {
    pub a_type: usize,
    pub a_val: usize
}

pub unsafe fn from_raw (auxv: *const Auxv) -> Vec<usize> {
    let mut aux: Vec<usize> = vec![0; AUX_CNT];
    let mut i: isize = 0;
    while (&*auxv.offset(i)).a_val != AT_NULL {
        let auxv_t = &*auxv.offset(i);
        // musl wants the aux a_val array to be indexed by AT_<TYPE>
        aux[auxv_t.a_type] = auxv_t.a_val;
        i += 1;
    }
    aux
}

#[inline(always)]
fn str_of_idx (idx: usize) -> &'static str {
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

pub fn show (auxvs: &Vec<usize>) {
    for (i, auxv) in auxvs.iter().enumerate() {
        if *auxv != 0 {
            println!("{}: 0x{:x}", str_of_idx(i), auxv)
        }
    }
}
