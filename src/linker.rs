use std::collections::HashMap;
use std::boxed::Box;
use std::fmt;

use binary::elf::header;
use binary::elf::program_header;
use binary::elf::dyn;

use kernel_block;
use image::elf::ElfExec;

use relocate;

pub struct Linker<'a> {
    pub base: u64,
    pub load_bias: u64,
    pub ehdr: &'a header::Header,
    pub phdrs: &'a [program_header::ProgramHeader],
    pub dynamic: &'a [dyn::Dyn],
    working_set: Box<HashMap<String, isize>>,
}

struct LinkInfo {
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
}

#[inline]
fn compute_load_bias(base:u64, phdrs:&[program_header::ProgramHeader]) -> u64 {
    for phdr in phdrs {
        if phdr.p_type == program_header::PT_LOAD {
            return base + (phdr.p_offset - phdr.p_vaddr);
        }
    }
    0
}

#[inline]
fn prelink(bias: u64, dynamic: &[dyn::Dyn]) -> LinkInfo {
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
    for dyn in dynamic {
        match dyn.d_tag {
            dyn::DT_RELA => rela = dyn.d_val + bias,
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
            dyn::DT_JMPREL => jmprel = dyn.d_val + bias,
            dyn::DT_VERNEED => verneed = dyn.d_val + bias,
            dyn::DT_VERNEEDNUM => verneednum = dyn.d_val,
            dyn::DT_VERSYM => versym = dyn.d_val + bias,
            dyn::DT_INIT => init = dyn.d_val + bias,
            dyn::DT_FINI => fini = dyn.d_val + bias,
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
    }
}

impl fmt::Debug for LinkInfo {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rela: 0x{:x} relasz: {} relaent: {} relacount: {} gnu_hash: 0x{:x} hash: 0x{:x} strtab: 0x{:x} strsz: {} symtab: 0x{:x} syment: {} pltgot: 0x{:x} pltrelsz: {} pltrel: {} jmprel: 0x{:x} verneed: 0x{:x} verneednum: {} versym: 0x{:x} init: 0x{:x} fini: 0x{:x}",
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
               self.fini
               )
    }
}

// this comes from musl
extern {
    fn __init_tls(aux: *const u64); // pointer to aux vector indexed by AT_<TYPE> that musl likes
}

impl<'a> Linker<'a> {
    pub fn new<'b> (base: u64, block: &kernel_block::KernelBlock) -> Result<Linker<'b>, &'static str> {
        unsafe {
            let ehdr = header::as_header(base as *const u64);
            let addr = (base + ehdr.e_phoff) as *const program_header::ProgramHeader;
            let phdrs = program_header::to_phdr_array(addr, ehdr.e_phnum as usize);
            let load_bias = compute_load_bias(base, &phdrs);

            if let Some(dynamic) = dyn::get_dynamic_array(load_bias, &phdrs) {
                let relocations = relocate::get_relocations(load_bias, &dynamic);
                relocate::relocate(load_bias, &relocations);
                // dryad has successfully relocated itself; time to init tls
                __init_tls(block.get_aux().as_ptr()); // this might not be safe yet because vec allocates
                
                Ok(Linker {
                    base: base,
                    load_bias: load_bias,
                    ehdr: &ehdr,
                    phdrs: &phdrs,
                    dynamic: &dynamic,
                    working_set: Box::new(HashMap::new()) // we relocated ourselves so it should be safe to heap allocate
                })
            } else {
                Err("<dryad> SEVERE: no dynamic array found for dryad; exiting\n")
            }
        }
    }

    pub fn link(&self, image: ElfExec) -> Result<(), String> {
        if let Some(dynamic) = image.dynamic {
            let link_info = prelink(image.load_bias, dynamic);
            println!("LinkInfo:\n  {:#?}", &link_info);
            Ok(())
        } else {
            Err(format!("<dryad> Main executable {} contains no _DYNAMIC", "TODO ADD EXE image.name"))
        }
    }

}
