use std::fmt;

use utils::*;

use binary::elf::program_header;
use binary::elf::program_header::ProgramHeader;
use binary::elf::dyn;
use binary::elf::dyn::Dyn;

// TODO: add needed vector
// TODO: add symtab &'a [Sym] instead of u64 ?
pub struct LinkInfo<'a> {
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
    pub libs: Vec<&'a str>
}

impl<'a> LinkInfo<'a> {
    pub fn new<'b>(bias: u64, dynamic: &'b [dyn::Dyn]) -> LinkInfo<'b> {
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
        let mut needed = vec![0; 20];
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
                dyn::DT_NEEDED => {
                    // this is a _dubiously_ premature optimization for likely a list of 3-4 libs
                    let len = needed.len();
                    if needed_count >= len {
                        needed.resize(len * 2, 0);
                    }
                    needed[needed_count] = dyn.d_val;
                    needed_count += 1;
                },
                _ => ()
            }
        }

        let mut libs:Vec<&str> = Vec::with_capacity(needed_count as usize);
        for i in 0..needed_count as usize {
            libs.push(str_at(strtab as *const u8, needed[i] as isize));
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
            libs: libs
        }
    }
}

impl<'a> fmt::Debug for LinkInfo<'a> {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rela: 0x{:x} relasz: {} relaent: {} relacount: {} gnu_hash: 0x{:x} hash: 0x{:x} strtab: 0x{:x} strsz: {} symtab: 0x{:x} syment: {} pltgot: 0x{:x} pltrelsz: {} pltrel: {} jmprel: 0x{:x} verneed: 0x{:x} verneednum: {} versym: 0x{:x} init: 0x{:x} fini: 0x{:x} libs: {:#?}",
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
               self.libs
               )
    }
}

pub struct Executable<'a, 'b> {
    pub name: &'b str,
    pub base: u64,
    pub load_bias: u64,
    pub phdrs: &'a[ProgramHeader],
    pub dynamic: &'a[Dyn],
    pub link_info: LinkInfo<'a>,
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

                /*
                let strtab = dyn::get_strtab(load_bias, dynamic);
                let needed = dyn::get_needed(dynamic, strtab, base, load_bias);
                 */

                Ok (Executable {
                    name: name,
                    base: base,
                    load_bias: load_bias,
                    phdrs: phdrs,
                    dynamic: dynamic,
                    link_info: link_info,
                })
            } else {
                Err (format!("<dryad> Error: executable {} has no _DYNAMIC array", name))
            }
        }
    }
}

impl<'a, 'b> fmt::Debug for Executable<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "name: {} base: {:x} load_bias: {:x}\n  ProgramHeaders: {:#?}\n  _DYNAMIC: {:#?}\n  LinkInfo: {:#?}",
               self.name, self.base, self.load_bias, self.phdrs, self.dynamic, self.link_info)
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
