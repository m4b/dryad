/// According to glibc:
/// ```c
/// /* The x86-64 never uses Elf64_Rel/Elf32_Rel relocations.  */
/// #define ELF_MACHINE_NO_REL 1
/// #define ELF_MACHINE_NO_RELA 0
/// ```
use std::fmt;

#[cfg(target_arch = "x86_64")]
pub use goblin::elf64 as elf;

#[cfg(target_arch = "x86")]
pub use goblin::elf32 as elf;

use elf::header::Header;
use elf::program_header::{self, ProgramHeader};
use elf::dyn::{self, Dyn};
use elf::sym::{self, Sym};
use elf::strtab::Strtab;
use elf::rela::{self, Rela};
use elf::gnu_hash::GnuHash;
use tls;

/// Computes the "load bias", which is normally the base.  However, in older Linux kernel's, 3.13, and whatever runs on travis, I have discovered that the kernel incorrectly maps the vdso with "bad" values.
///
/// Specifically, I'm seeing the `p_vaddr` value and `d_val` in the dynamic array showing up with overflow biased values, e.g.:
/// ```
/// p/x phdr.p_vaddr
/// $1 = 0xffffffffff700000
/// (gdb) p/x bias
/// $2 = 0x00007fff873f1000
/// ```
/// In newer kernels the `phdr.p_vaddr` (correctly) reports `0x468` (it's a `ET_DYN` after all), which is then safely added to the original bias/load address to recover the desired address in memory, in this case: 0x7ffff7ff7468.
/// Unfortunately, life is not so easy in 3.13, as we're told the `p_vaddr` (and the `d_val` in the dynamic entry entries) is a crazy value like above.  Luckily, after looking at what several dynamic linkers do, I noticed that they all seem to implement a version of the following, in that we can recover the correct address by relying on insane overflow arithmetic _regardless_ of whether we received this crazy address, or a normal address:
/// ```
/// load_bias = base - p_vaddr + p_offset
/// ```
/// In the 3.13 case:
/// ```
///    let load_bias = 0x7fff873f1000u64.wrapping_sub(0xffffffffff700000u64.wrapping_add(0));
///    println!("load_bias: 0x{:x}", load_bias);
///    let dynamic = load_bias.wrapping_add(0xffffffffff700468);
///    println!("dynamic: 0x{:x}", dynamic);
/// ```
/// On my machine with `4.4.5-1-ARCH`, the computed load bias will equal itself, and subsequent additions of sane `p_vaddr` values will work as expected.
/// As for why the above is the case on older kernels (or perhaps VMs only, I haven't tested extensively), I have no idea.
#[inline(always)]
pub unsafe fn compute_load_bias_wrapping(base: u64, phdrs:&[ProgramHeader]) -> usize {
    for phdr in phdrs {
        if phdr.p_type == program_header::PT_LOAD {
            return base.wrapping_sub(phdr.p_vaddr.wrapping_add(phdr.p_offset)) as usize;
        }
    }
    0
}

/// A `SharedObject` is either:
/// 1. an mmap'd dynamic library which is explicitly loaded by `dryad`
/// 2. the vdso provided by the kernel
/// 3. the executable we're interpreting
pub struct SharedObject<'process> {
    pub name: &'process str,
    pub load_bias: u64, // TODO: change to usize change this to addr or base_addr load_bias is stupid
    pub map_begin: u64, // probably remove these?
    pub map_end: u64,
    pub libs: Vec<&'process str>,
    pub phdrs: &'process[ProgramHeader],
    pub dynamic: &'process [Dyn],
    pub strtab: Strtab<'process>,
    pub symtab: &'process[Sym],
    pub relatab: &'process[Rela],
    pub pltrelatab: &'process[Rela],
    pub pltgot: *const u64,
    pub gnu_hash: Option<GnuHash<'process>>,
    pub load_path: Option<String>,
    pub flags: u64,
    pub state_flags: u64,
    pub tls: Option<tls::TlsInfo>,
    pub link_info: dyn::DynamicInfo,
}

impl<'process> fmt::Debug for SharedObject<'process> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name: {} load_bias: {:x} DT_FLAGS: 0x{:x} DT_FLAGS_1 0x{:x}\n  ProgramHeaders: {:#?}\n  _DYNAMIC: {:#?}\n  String Table: {:#?}\n  Symbol Table: {:#?}\n  Rela Table: {:#?}\n  Plt Rela Table: {:#?}\n  Libraries: {:#?}\n",
               self.name, self.load_bias, self.flags, self.state_flags, self.phdrs, self.dynamic, self.strtab, self.symtab, self.relatab, self.pltrelatab, self.libs)
    }
}

impl<'process> SharedObject<'process> {

    /// Assumes the object referenced by the ptr has already been mmap'd or loaded into memory some way
    pub unsafe fn from_raw (ptr: u64) -> SharedObject<'process> {
        let header = &*(ptr as *const Header);
        let phdrs = ProgramHeader::from_raw_parts((header.e_phoff + ptr) as *const ProgramHeader, header.e_phnum as usize);
        let load_bias = compute_load_bias_wrapping(ptr, &phdrs);
        let dynamic = dyn::from_phdrs(load_bias as u64, phdrs).unwrap();
        let link_info = dyn::DynamicInfo::new(&dynamic, load_bias);
        let num_syms = (link_info.strtab - link_info.symtab) / sym::SIZEOF_SYM;
        let symtab = sym::from_raw(link_info.symtab as *const sym::Sym, num_syms);
        let strtab = Strtab::from_raw(link_info.strtab as *const u8, link_info.strsz as usize);
        let libs = dyn::get_needed(dynamic, &strtab, link_info.needed_count);
        let relatab = rela::from_raw(link_info.rela as *const rela::Rela, link_info.relasz);
        let pltrelatab = rela::from_raw(link_info.jmprel as *const rela::Rela, link_info.pltrelsz);
        let pltgot = if let Some(addr) = link_info.pltgot { addr } else { 0 };
        let gnu_hash = if let Some(addr) = link_info.gnu_hash { Some (GnuHash::new(addr as *const u32, symtab.len())) } else { None };
        SharedObject {
            name: strtab.get(link_info.soname),
            load_bias: ptr,
            map_begin: 0,
            map_end: 0,
            libs: libs,
            phdrs: phdrs,
            dynamic: dynamic,
            symtab: symtab,
            strtab: strtab,
            relatab: relatab,
            pltrelatab: pltrelatab,
            pltgot: pltgot as *const u64,
            gnu_hash: gnu_hash,
            load_path: None,
            flags: link_info.flags,
            state_flags: link_info.flags_1,
            tls: None, // TODO: should probably check for tls, even tho this currently only used for linux gate
            link_info: link_info,
        }

    }

    pub fn from_executable (name: &'static str, phdr_addr: u64, phnum: usize, lachesis: &mut tls::Lachesis) -> Result<SharedObject<'process>, String> {
        unsafe {
            let addr = phdr_addr as *const ProgramHeader;
            let phdrs = ProgramHeader::from_raw_parts(addr, phnum);

            let mut load_bias = 0;
            let mut dynamic_vaddr = None;
            let mut tls_phdr = None;
            for phdr in phdrs {
                match phdr.p_type {
                    program_header::PT_PHDR => {
                        load_bias = phdr_addr - phdr.p_vaddr;
                    },
                    program_header::PT_DYNAMIC => {
                        dynamic_vaddr = Some(phdr.p_vaddr);
                    },
                    program_header::PT_TLS => {
                        tls_phdr = Some(phdr);
                    },
                    _ => ()
                }
            }

            let tls = if let Some(phdr) = tls_phdr {
                Some(lachesis.push_module(name, load_bias as usize, phdr))
            } else { None };

            if let Some(vaddr) = dynamic_vaddr {
                let dynamic = dyn::from_raw(load_bias, vaddr);
                let link_info = dyn::DynamicInfo::new(dynamic, load_bias as usize);
                // TODO: swap out the link_info syment with compile time constant SIZEOF_SYM?
                let num_syms = (link_info.strtab - link_info.symtab) / link_info.syment; // this _CAN'T_ generally be valid; but rdr has been doing it and scans every linux shared object binary without issue... so it must be right!
                let symtab = sym::from_raw(link_info.symtab as *const sym::Sym, num_syms);
                let strtab = Strtab::from_raw(link_info.strtab as *const u8, link_info.strsz);
                let libs = dyn::get_needed(dynamic, &strtab, link_info.needed_count);
                let relatab = rela::from_raw(link_info.rela as *const rela::Rela, link_info.relasz);
                let pltrelatab = rela::from_raw(link_info.jmprel as *const rela::Rela, link_info.pltrelsz);

                // TODO: fail with Err, not panic
                let pltgot = link_info.pltgot.expect("Error executable has no pltgot, aborting") as *const u64;
                let gnu_hash = if let Some(addr) = link_info.gnu_hash { Some (GnuHash::new(addr as *const u32, symtab.len())) } else { None };
                Ok (SharedObject {
                    name: name,
                    load_bias: load_bias,
                    map_begin: 0,
                    map_end: 0,
                    libs: libs,
                    phdrs: phdrs,
                    dynamic: dynamic,
                    symtab: symtab,
                    strtab: strtab,
                    relatab: relatab,
                    pltrelatab: pltrelatab,
                    pltgot: pltgot,
                    gnu_hash: gnu_hash,
                    load_path: Some (name.to_string()), // TODO: make absolute?,
                    flags: link_info.flags,
                    state_flags: link_info.flags_1,
                    tls: tls,
                    link_info: link_info,
                })

            } else {
                Err (format!("Error: executable {} has no _DYNAMIC array", name))
            }
        }
    }

    /// This is used by dryad's runtime symbol resolution
    pub fn find (&self, name: &str, hash: u32) -> Option<sym::Sym> {
//        println!("<{}.find> finding symbol: {}", self.name, symbol);
        match self.gnu_hash {
            Some (ref gnu_hash) => gnu_hash.find(name, hash, &self.strtab, &self.symtab),
            None => None
        }
    }

}

//unsafe impl<'a> Send for SharedObject<'a> {}
//unsafe impl<'a> Sync for SharedObject<'a> {}
