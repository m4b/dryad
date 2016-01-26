use std::fmt;

use utils::*;

use binary::elf::program_header;
use binary::elf::program_header::ProgramHeader;
use binary::elf::dyn;
use binary::elf::dyn::Dyn;

pub struct LinkInfo {
    pub rela:u64,
    pub relasz:u64,
    pub relaent:u64,
    pub relacount:u64,
    pub gnu_hash:u64,
    pub hash:u64,
    pub strtab:u64,
    pub strsz:u64,
    pub symtab:u64,
    pub syment:u64,
    pub pltgot:u64,
    pub pltrelsz:u64,
    pub pltrel:u64,
    pub jmprel:u64,
    pub verneed:u64,
    pub verneednum:u64,
    pub versym:u64,
    pub init:u64,
    pub fini:u64,
    pub needed_count:usize,
}

impl LinkInfo {
    pub fn new(bias: u64, dynamic: &[dyn::Dyn]) -> LinkInfo {
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
        let mut needed_count = 0;
        for dyn in dynamic {
            match dyn.d_tag {
                dyn::DT_RELA => rela = dyn.d_val + bias, // .rela.dyn
                dyn::DT_RELASZ => relasz = dyn.d_val,
                dyn::DT_RELAENT => relaent = dyn.d_val,
                dyn::DT_RELACOUNT => relacount = dyn.d_val,
                dyn::DT_GNU_HASH => gnu_hash = dyn.d_val + bias,
                dyn::DT_HASH => hash = dyn.d_val + bias,
                dyn::DT_STRTAB => strtab = dyn.d_val + bias,
                dyn::DT_STRSZ => strsz = dyn.d_val,
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
                dyn::DT_NEEDED => needed_count += 1,
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
            needed_count: needed_count,
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

pub struct Executable<'a, 'b> {
    pub name: &'b str,
    pub base: u64,
    pub load_bias: u64,
    pub phdrs: &'a[ProgramHeader],
    pub dynamic: &'a[Dyn],
    pub link_info: LinkInfo,
    pub needed: Vec<&'a str>,
}

impl<'a, 'a2> Executable<'a, 'a2> {
    pub fn new<'b, 'c> (name: &'c str, phdr_addr: u64, phnum: usize) -> Result<Executable<'b, 'c>, String> {
        unsafe {
            let addr = phdr_addr as *const ProgramHeader;
            let phdrs = program_header::to_phdr_array(addr, phnum);
            let mut base = 0;
            let mut load_bias = 0;

            for phdr in phdrs {
                if phdr.p_type == program_header::PT_PHDR {
                    load_bias = phdr_addr - phdr.p_vaddr;
                    base = phdr_addr - phdr.p_offset;
                    break;
                }
            }
            // if base == 0 then no PT_PHDR and we should terminate? or kernel should have noticed this and we needn't bother

            if let Some(dynamic) = dyn::get_dynamic_array(load_bias, phdrs) {

                let link_info = LinkInfo::new(load_bias, dynamic);
                let needed = dyn::get_needed(dynamic, link_info.strtab, load_bias, link_info.needed_count);
                /*
                let strtab = dyn::get_strtab(load_bias, dynamic);
                 */

                Ok (Executable {
                    name: name,
                    base: base,
                    load_bias: load_bias,
                    phdrs: phdrs,
                    dynamic: dynamic,
                    link_info: link_info,
                    needed: needed,
                })
            } else {

                Err (format!("<dryad> Error: executable {} has no _DYNAMIC array", name))

            }
        }
    }
}

impl<'a, 'b> fmt::Debug for Executable<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name: {} base: {:x} load_bias: {:x}\n  ProgramHeaders: {:#?}\n  _DYNAMIC: {:#?}\n  LinkInfo: {:#?}\n  Needed: {:#?}",
               self.name, self.base, self.load_bias, self.phdrs, self.dynamic, self.link_info, self.needed)
    }
}

/// A SharedObject is an mmap'd dynamic library
pub struct SharedObject {
    pub name: String,
    pub phdrs: Vec<ProgramHeader>,
    pub dynamic: Vec<Dyn>,
    pub base: u64,
    pub load_bias: u64,
    pub libs: Vec<String>,
}