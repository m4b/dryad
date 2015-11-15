use binary::elf::header;
use binary::elf::program_header;
use binary::elf::dyn;

use relocate;

pub struct Linker<'a> {
    pub base: u64,
    pub load_bias: u64,
    pub ehdr: &'a header::Header,
    pub phdrs: &'a [program_header::ProgramHeader],
    pub dynamic: &'a [dyn::Dyn],
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

impl<'a> Linker<'a> {
    pub fn new<'b> (base: u64) -> Result<Linker<'b>, &'static str> {
        unsafe {
            let ehdr = header::as_header(base as *const u64);
            let addr = (base + ehdr.e_phoff) as *const program_header::ProgramHeader;
            let phdrs = program_header::to_phdr_array(addr, ehdr.e_phnum as usize);
            let load_bias = compute_load_bias(base, &phdrs);
            if let Some(dynamic) = dyn::get_dynamic_array(load_bias, &phdrs) {
                let relocations = relocate::get_relocations(load_bias, &dynamic);
                relocate::relocate(load_bias, &relocations);
                Ok(Linker {
                    base: base,
                    load_bias: load_bias,
                    ehdr: &ehdr,
                    phdrs: &phdrs,
                    dynamic: &dynamic
                })
            } else {
                Err("<dryad> SEVERE: no dynamic array found for dryad; exiting\n")
            }
        }
    }
}
