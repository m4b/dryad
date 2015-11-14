pub mod elf {

    use kernel_block;
    use auxv;
    use utils::*;

    // lore:
    //<aatch> It should work, if you do `$t { $($fld: $fld),+ }`.
    //<Gankro> ^--- why I hate Rust's macros // [22:21]

    //<durka42> macro_rules! init { ($t:ident: $($fld:ident),* $(,)*) => { $t {   $($fld: $fld),* } } }
    macro_rules! init {
        ($t: ident: $($fld: ident,)+) => { $t { $($fld: $fld),* } };
        ($t: ident: $($fld: ident),+) => { $t { $($fld: $fld),* } }
    }
    // usage: init! { Elf: base, phdr, phent, phnum, entry }

    pub struct Elf {
        pub base: u64,
        pub phdr: u64,
        pub phent: u64, // sizeof an elf program header entry
        pub phnum: u64,
        pub entry: u64,
        /*
        pub load_bias: u64,
        pub dynamic: * const u64, // future: *const elf_dyn
        pub strtab: * const u8,
        pub symtab: * const u64, // future: *const elf_sym
         */
    }

    impl Elf {
        pub fn new (block: &kernel_block::KernelBlock) -> Elf {
            // base is also = elf_hdr
            let base   = block.getauxval(auxv::AT_BASE).unwrap();
            let phdr   = block.getauxval(auxv::AT_PHDR).unwrap();
            let phent  = block.getauxval(auxv::AT_PHENT).unwrap();
            let phnum  = block.getauxval(auxv::AT_PHNUM).unwrap();
            let entry  = block.getauxval(auxv::AT_ENTRY).unwrap();
            Elf {
                base: base,
                phdr: phdr,
                phent: phent,
                phnum: phnum,
                entry: entry
            }
        }
        pub unsafe fn debug_print (&self) {
            write(&"base: 0x");
            write_u64(self.base, true);
            write(&"\n");
            write(&"phdr: 0x");
            write_u64(self.phdr as u64, true);
            write(&"\n");
            write(&"phent: ");
            write_u64(self.phent, false);
            write(&"\n");
            write(&"phnum: ");
            write_u64(self.phnum, false);
            write(&"\n");
            write(&"entry: 0x");
            write_u64(self.entry, true);
            write(&"\n");
        }
    }
}
