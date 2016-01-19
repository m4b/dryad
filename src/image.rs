pub mod elf {

    use std::fmt;
    
//    use binary::elf::header::Header;
    use binary::elf::program_header;
    use binary::elf::program_header::ProgramHeader;
    use binary::elf::dyn;
    use binary::elf::dyn::Dyn;

    pub struct ElfExec<'a, 'b> {
        pub name: &'b str,
        pub base: u64,
        pub load_bias: u64,
        pub phdrs: &'a[ProgramHeader],
        pub dynamic: Option<&'a[Dyn]>,
    }

    pub struct SharedObject<'a> {
        pub phdrs: &'a[ProgramHeader],
        pub dynamic: &'a[Dyn],
        pub base: u64,
        pub load_bias: u64,
    }

    impl<'a, 'a2> ElfExec<'a, 'a2> {
        pub fn new<'b, 'c> (name: &'c str, phdr_addr: u64, phnum: usize) -> ElfExec<'b, 'c> {
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

                let dynamic = dyn::get_dynamic_array(load_bias, phdrs);
                /*
                let strtab = dyn::get_strtab(load_bias, dynamic);
                let needed = dyn::get_needed(dynamic, strtab, base, load_bias);
                 */
                
                ElfExec {
                    name: name,
                    base: base,
                    load_bias: load_bias,
                    phdrs: phdrs,
                    dynamic: dynamic
                }
            }
        }
    }

    impl<'a, 'b> fmt::Debug for ElfExec<'a, 'b> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "name: {} base: {:x} load_bias: {:x}\n  ProgramHeaders: {:#?}\n  _DYNAMIC: {:#?}",
                   self.name, self.base, self.load_bias, self.phdrs, self.dynamic)
        }
    }
}
