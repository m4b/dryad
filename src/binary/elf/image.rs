/// TODO: decide on whether to support only Rela (probably yes?); have rdr scan binaries to see frequency of rel (no addend) relocation tables
use std::fmt;

use binary::elf::header::Header;
use binary::elf::program_header;
use binary::elf::program_header::ProgramHeader;
use binary::elf::dyn;
use binary::elf::dyn::Dyn;
use binary::elf::sym;
use binary::elf::sym::Sym;
use binary::elf::strtab::Strtab;
use binary::elf::rela;
use binary::elf::rela::Rela;
use binary::elf::gnu_hash::GnuHash;

/// Important dynamic LinkInfo generated via a single pass through the _DYNAMIC array
pub struct LinkInfo {
    pub rela: u64,
    pub relasz: u64, // TODO: make this a usize?
    pub relaent: u64,
    pub relacount: u64,
    pub gnu_hash: u64,
    pub hash: u64,
    pub strtab: u64,
    pub strsz: usize,
    pub symtab: u64,
    pub syment: u64,
    pub pltgot: u64,
    pub pltrelsz: u64,
    pub pltrel: u64,
    pub jmprel: u64,
    pub verneed: u64,
    pub verneednum: u64,
    pub versym: u64,
    pub init: u64,
    pub fini: u64,
    pub init_array: u64,
    pub init_arraysz: usize,
    pub fini_array: u64,
    pub fini_arraysz: usize,
    pub needed_count: usize,
    pub flags: u64,
    pub flags_1: u64,
    pub soname: usize,
}

impl LinkInfo {
    pub fn new(dynamic: &[dyn::Dyn], bias: u64) -> LinkInfo {
        let mut rela = 0;
        let mut relasz = 0;
        let mut relaent = 0;
        let mut relacount = 0;
        let mut gnu_hash = 0;
        let mut hash = 0;
        let mut strtab = 0;
        let mut strsz = 0;
        let mut symtab = 0;
        let mut syment = 0;
        let mut pltgot = 0;
        let mut pltrelsz = 0;
        let mut pltrel = 0;
        let mut jmprel = 0;
        let mut verneed = 0;
        let mut verneednum = 0;
        let mut versym = 0;
        let mut init = 0;
        let mut fini = 0;
        let mut init_array = 0;
        let mut init_arraysz = 0;
        let mut fini_array = 0;
        let mut fini_arraysz = 0;
        let mut needed_count = 0;
        let mut flags = 0;
        let mut flags_1 = 0;
        let mut soname = 0;
        for dyn in dynamic {
            match dyn.d_tag {
                dyn::DT_RELA => rela = dyn.d_val + bias, // .rela.dyn
                dyn::DT_RELASZ => relasz = dyn.d_val,
                dyn::DT_RELAENT => relaent = dyn.d_val,
                dyn::DT_RELACOUNT => relacount = dyn.d_val,
                dyn::DT_GNU_HASH => gnu_hash = dyn.d_val + bias,
                dyn::DT_HASH => hash = dyn.d_val + bias,
                dyn::DT_STRTAB => strtab = dyn.d_val + bias,
                dyn::DT_STRSZ => strsz = dyn.d_val as usize,
                dyn::DT_SYMTAB => symtab = dyn.d_val + bias,
                dyn::DT_SYMENT => syment = dyn.d_val,
                dyn::DT_PLTGOT => pltgot = dyn.d_val + bias,
                dyn::DT_PLTRELSZ => pltrelsz = dyn.d_val,
                dyn::DT_PLTREL => pltrel = dyn.d_val,
                dyn::DT_JMPREL => jmprel = dyn.d_val + bias, // .rela.plt
                dyn::DT_VERNEED => verneed = dyn.d_val + bias,
                dyn::DT_VERNEEDNUM => verneednum = dyn.d_val,
                dyn::DT_VERSYM => versym = dyn.d_val + bias,
                dyn::DT_INIT => init = dyn.d_val + bias,
                dyn::DT_FINI => fini = dyn.d_val + bias,
                dyn::DT_INIT_ARRAY => init_array = dyn.d_val + bias,
                dyn::DT_INIT_ARRAYSZ => init_arraysz = dyn.d_val,
                dyn::DT_FINI_ARRAY => fini_array = dyn.d_val + bias,
                dyn::DT_FINI_ARRAYSZ => fini_arraysz = dyn.d_val,
                dyn::DT_NEEDED => needed_count += 1,
                dyn::DT_FLAGS => flags = dyn.d_val,
                dyn::DT_FLAGS_1 => flags_1 = dyn.d_val,
                dyn::DT_SONAME => soname = dyn.d_val,
                _ => ()
            }
        }

        LinkInfo {
            rela: rela,
            relasz: relasz,
            relaent: relaent,
            relacount: relacount,
            gnu_hash: gnu_hash,
            hash: hash,
            strtab: strtab,
            strsz: strsz,
            symtab: symtab,
            syment: syment,
            pltgot: pltgot,
            pltrelsz: pltrelsz,
            pltrel: pltrel,
            jmprel: jmprel,
            verneed: verneed,
            verneednum: verneednum,
            versym: versym,
            init: init,
            fini: fini,
            init_array: init_array,
            init_arraysz: init_arraysz as usize,
            fini_array: fini_array,
            fini_arraysz: fini_arraysz as usize,
            needed_count: needed_count,
            flags: flags,
            flags_1: flags_1,
            soname: soname as usize,
        }
    }
}

impl fmt::Debug for LinkInfo {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rela: 0x{:x} relasz: {} relaent: {} relacount: {} gnu_hash: 0x{:x} hash: 0x{:x} strtab: 0x{:x} strsz: {} symtab: 0x{:x} syment: {} pltgot: 0x{:x} pltrelsz: {} pltrel: {} jmprel: 0x{:x} verneed: 0x{:x} verneednum: {} versym: 0x{:x} init: 0x{:x} fini: 0x{:x} needed_count: {}",
               self.rela,
               self.relasz,
               self.relaent,
               self.relacount,
               self.gnu_hash,
               self.hash,
               self.strtab,
               self.strsz,
               self.symtab,
               self.syment,
               self.pltgot,
               self.pltrelsz,
               self.pltrel,
               self.jmprel,
               self.verneed,
               self.verneednum,
               self.versym,
               self.init,
               self.fini,
               self.needed_count,
               )
    }
}

/// A `SharedObject` is either:
/// 1. an mmap'd dynamic library which is explicitly loaded by `dryad`
/// 2. the vdso provided by the kernel
/// 3. the executable we're interpreting
pub struct SharedObject<'process> {
    pub name: String,
    pub load_bias: u64,
    pub map_begin: u64,
    pub map_end: u64,
    pub libs: Vec<&'process str>,
    pub phdrs: Vec<ProgramHeader>, // todo remove the vec
    pub dynamic: &'process[Dyn],
    pub strtab: Strtab<'process>,
    pub symtab: &'process[Sym],
    pub relatab: &'process[Rela],
    pub pltrelatab: &'process[Rela],
    pub pltgot: *const u64,
    pub gnu_hash: GnuHash<'process>
}

impl<'process> fmt::Debug for SharedObject<'process> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name: {} load_bias: {:x}\n  ProgramHeaders: {:#?}\n  _DYNAMIC: {:#?}\n  String Table: {:#?}\n  Symbol Table: {:#?}\n  Rela Table: {:#?}\n  Plt Rela Table: {:#?}\n  Libraries: {:#?}",
               self.name, self.load_bias, self.phdrs, self.dynamic, self.strtab, self.symtab, self.relatab, self.pltrelatab, self.libs)
    }
}

impl<'process> SharedObject<'process> {

    /// Assumes the object referenced by the ptr has already been mmap'd or loaded into memory some way
    /// TODO: fix the libs hack
    pub unsafe fn from_raw (ptr: u64) -> SharedObject<'process> {
        let header = &*(ptr as *const Header);
        let phdrs = ProgramHeader::from_raw_parts((header.e_phoff + ptr) as *const ProgramHeader, header.e_phnum as usize);
        let dynamic = dyn::get_dynamic_array(ptr as u64, phdrs).unwrap();
        let link_info = LinkInfo::new(&dynamic, ptr);
        let num_syms = (link_info.strtab - link_info.symtab) / sym::SIZEOF_SYM as u64;
        let symtab = sym::get_symtab(link_info.symtab as *const sym::Sym, num_syms as usize);
        let strtab = Strtab::new(link_info.strtab as *const u8, link_info.strsz as usize);
        let libs = dyn::get_needed(dynamic, &strtab, link_info.needed_count);
        let soname = strtab[link_info.soname].to_string(); // TODO: remove this allocation
        let relatab = rela::get(link_info.rela, link_info.relasz as usize, link_info.relaent as usize, link_info.relacount as usize);
        let pltrelatab = rela::get_plt(link_info.jmprel, link_info.pltrelsz as usize);
        let gnu_hash = GnuHash::new(link_info.gnu_hash as *const u32, symtab.len());
        SharedObject {
            name: soname,
            load_bias: ptr,
            map_begin: 0,
            map_end: 0,
            libs: libs,
            phdrs: phdrs.to_owned(),
            dynamic: dynamic,
            symtab: symtab,
            strtab: strtab,
            relatab: relatab,
            pltrelatab: pltrelatab,
            pltgot: link_info.pltgot as *const u64,
            gnu_hash: gnu_hash,
        }
    }

    pub fn from_executable (name: &'static str, phdr_addr: u64, phnum: usize) -> Result<SharedObject<'process>, String> {
        unsafe {
            let addr = phdr_addr as *const ProgramHeader;
            let phdrs = ProgramHeader::from_raw_parts(addr, phnum);
//            let mut base = 0;
            let mut load_bias = 0;

            for phdr in phdrs {
                if phdr.p_type == program_header::PT_PHDR {
                    load_bias = phdr_addr - phdr.p_vaddr;
//                    base = phdr_addr - phdr.p_offset;
                    break;
                }
            }
            // if base == 0 then no PT_PHDR and we should terminate? or kernel should have noticed this and we needn't bother

            if let Some(dynamic) = dyn::get_dynamic_array(load_bias, phdrs) {

                let link_info = LinkInfo::new(dynamic, load_bias);
                // TODO: swap out the link_info syment with compile time constant SIZEOF_SYM?
                let num_syms = ((link_info.strtab - link_info.symtab) / link_info.syment) as usize; // this _CAN'T_ generally be valid; but rdr has been doing it and scans every linux shared object binary without issue... so it must be right!
                let symtab = sym::get_symtab(link_info.symtab as *const sym::Sym, num_syms);
                let strtab = Strtab::new(link_info.strtab as *const u8, link_info.strsz);
                let libs = dyn::get_needed(dynamic, &strtab, link_info.needed_count);
                let relatab = rela::get(link_info.rela, link_info.relasz as usize, link_info.relaent as usize, link_info.relacount as usize);
                let pltrelatab = rela::get_plt(link_info.jmprel, link_info.pltrelsz as usize);

                let pltgot = link_info.pltgot as *const u64;

                Ok (SharedObject {
                    name: name.to_string(),
                    load_bias: load_bias,
                    map_begin: 0,
                    map_end: 0,
                    libs: libs,
                    phdrs: phdrs.to_owned(),
                    dynamic: dynamic,
                    symtab: symtab,
                    strtab: strtab,
                    relatab: relatab,
                    pltrelatab: pltrelatab,
                    pltgot: pltgot,
                    gnu_hash: GnuHash::new(link_info.gnu_hash as *const u32, symtab.len()),
                })

            } else {

                Err (format!("<dryad> Error: executable {} has no _DYNAMIC array", name))
            }
        }
    }

    pub fn find (&self, name: &str, hash: u32) -> Option<u64> {
//        println!("<{}.find> finding symbol: {}", self.name, symbol);
        self.gnu_hash.find(name, hash, self)
    }
}

/*
unsafe impl<'a> Send for SharedObject<'a> {}
unsafe impl<'a> Sync for SharedObject<'a> {}
*/
