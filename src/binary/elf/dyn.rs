/// TODO: need to add the DF_1* and DF_* flags here...

use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom::Start;
use std::fmt;
use std::slice;
use std::mem;
use utils::*;
use binary::elf::program_header::{ ProgramHeader, PT_DYNAMIC };
use binary::elf::strtab::Strtab;

pub const DT_NULL: u64 = 0;
pub const DT_NEEDED: u64 = 1;
pub const DT_PLTRELSZ: u64 = 2;
pub const DT_PLTGOT: u64 = 3;
pub const DT_HASH: u64 = 4;
pub const DT_STRTAB: u64 = 5;
pub const DT_SYMTAB: u64 = 6;
pub const DT_RELA: u64 = 7;
pub const DT_RELASZ: u64 = 8;
pub const DT_RELAENT: u64 = 9;
pub const DT_STRSZ: u64 = 10;
pub const DT_SYMENT: u64 = 11;
pub const DT_INIT: u64 = 12;
pub const DT_FINI: u64 = 13;
pub const DT_SONAME: u64 = 14;
pub const DT_RPATH: u64 = 15;
pub const DT_SYMBOLIC: u64 = 16;
pub const DT_REL: u64 = 17;
pub const DT_RELSZ: u64 = 18;
pub const DT_RELENT: u64 = 19;
pub const DT_PLTREL: u64 = 20;
pub const DT_DEBUG: u64 = 21;
pub const DT_TEXTREL: u64 = 22;
pub const DT_JMPREL: u64 = 23;
pub const DT_BIND_NOW: u64 = 24;
pub const DT_INIT_ARRAY: u64 = 25;
pub const DT_FINI_ARRAY: u64 = 26;
pub const DT_INIT_ARRAYSZ: u64 = 27;
pub const DT_FINI_ARRAYSZ: u64 = 28;
pub const DT_RUNPATH: u64 = 29;
pub const DT_FLAGS: u64 = 30;
pub const DT_ENCODING: u64 = 32;
pub const DT_PREINIT_ARRAY:u32 = 32;
pub const DT_PREINIT_ARRAYSZ:u32 = 33;
pub const DT_NUM: u64 = 34;
pub const DT_LOOS: u64 = 0x6000000d;
pub const DT_HIOS: u64 = 0x6ffff000;
pub const DT_LOPROC: u64 = 0x70000000;
pub const DT_HIPROC: u64 = 0x7fffffff;
//pub const DT_PROCNUM: u64 = DT_MIPS_NUM;
pub const DT_VERSYM: u64 = 0x6ffffff0;
pub const DT_RELACOUNT: u64 = 0x6ffffff9;
pub const DT_RELCOUNT: u64 = 0x6ffffffa;

// new
pub const DT_GNU_HASH: u64 = 0x6ffffef5;
pub const DT_VERDEF: u64 = 0x6ffffffc;
pub const DT_VERDEFNUM: u64 = 0x6ffffffd;
pub const DT_VERNEED: u64 = 0x6ffffffe;
pub const DT_VERNEEDNUM: u64 = 0x6fffffff;
pub const DT_FLAGS_1: u64 = 0x6ffffffb;

/// An entry in the dynamic array
#[repr(C)]
#[derive(Clone)]
pub struct Dyn {
    pub d_tag: u64, // Dynamic entry type
    pub d_val: u64, // Integer value
}

pub const SIZEOF_DYN:usize = 16;

impl Dyn {
    unsafe fn debug_print(&self) {
        write(&"d_tag: 0x");
        write_u64(self.d_tag, true);
        write(&"\n");
        write(&"d_val: 0x");
        write_u64(self.d_val, true);
        write(&"\n");
    }
}

/// TODO: swap out nums with the consts
/// Converts a u64 tag to its string representation
#[inline]
fn tag_to_str(tag: u64) -> &'static str {
    match tag {
        0 => "DT_NULL",
        1 => "DT_NEEDED",
        2 => "DT_PLTRELSZ",
        3 => "DT_PLTGOT",
        4 => "DT_HASH",
        5 => "DT_STRTAB",
        6 => "DT_SYMTAB",
        7 => "DT_RELA",
        8 => "DT_RELASZ",
        9 => "DT_RELAENT",
        10 => "DT_STRSZ:",
        11 => "DT_SYMENT",
        12 => "DT_INIT",
        13 => "DT_FINI",
        14 => "DT_SONAME",
        15 => "DT_RPATH",
        16 => "DT_SYMBOLIC",
        17 => "DT_REL",
        18 => "DT_RELSZ",
        19 => "DT_RELENT",
        20 => "DT_PLTREL",
        21 => "DT_DEBUG",
        22 => "DT_TEXTREL",
        23 => "DT_JMPREL",
        24 => "DT_BIND_NOW",
        25 => "DT_INIT_ARRAY",
        26 => "DT_FINI_ARRAY",
        27 => "DT_INIT_ARRAYSZ",
        28 => "DT_FINI_ARRAYSZ",
        29 => "DT_RUNPATH",
        30 => "DT_FLAGS",
        32 => "DT_PREINIT_ARRAY",
        33 => "DT_PREINIT_ARRAYSZ",
        34 => "DT_NUM",
        0x6000000d => "DT_LOOS",
        0x6ffff000 => "DT_HIOS",
        0x70000000 => "DT_LOPROC",
        0x7fffffff => "DT_HIPROC",
        0x6ffffff0 => "DT_VERSYM",
        0x6ffffff9 => "DT_RELACOUNT",
        0x6ffffffa => "DT_RELCOUNT",
        // new
        0x6ffffef5 => "DT_GNU_HASH",
        0x6ffffffc => "DT_VERDEF",
        0x6ffffffd => "DT_VERDEFNUM",
        0x6ffffffe => "DT_VERNEED",
        0x6fffffff => "DT_VERNEEDNUM",
        DT_FLAGS_1 => "DT_FLAGS_1",
        _ => "UNKNOWN_TAG"
    }
}

impl fmt::Debug for Dyn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "d_tag: {} d_val: 0x{:x}", tag_to_str(self.d_tag), self.d_val)
    }
}

// this is broken, not to mention weirdness on the mut fd
// _BUT_: will be interesting to benchmark this against mmap-ing
pub fn from_fd<'a>(mut fd: &File, phdrs: &'a [ProgramHeader]) -> Option<Vec<Dyn>> {
    for phdr in phdrs {
        if phdr.p_type == PT_DYNAMIC {
            let filesz = phdr.p_filesz as usize;
            let dync = filesz / SIZEOF_DYN;
            let mut dyns: Vec<u8> = vec![0; filesz];
            let _ = fd.seek(Start(phdr.p_offset));
            let _ = fd.read(dyns.as_mut_slice());
            let dyns:Vec<Dyn> = unsafe { mem::transmute(dyns) };
            return Some (dyns)
        }
    }
    None
}

/// Maybe gets and returns the dynamic array with the same lifetime as the [phdrs], using the provided bias.
/// If the bias is wrong, it will either segfault or give you incorrect values, beware
pub unsafe fn get_dynamic_array<'a>(bias: u64, phdrs: &'a [ProgramHeader]) -> Option<&'a [Dyn]> {
    for phdr in phdrs {
        if phdr.p_type == PT_DYNAMIC {
            let dynp = (phdr.p_vaddr + bias) as *const Dyn;
            let mut idx = 0;
            while (*(dynp.offset(idx))).d_tag != DT_NULL {
                idx += 1;
            }
            return Some(slice::from_raw_parts(dynp, idx as usize))
        }
    }
    None
}

/// Gets the needed libraries from the `_DYNAMIC` array, with the str slices lifetime tied to the dynamic arrays lifetime
pub fn get_needed<'a, 'b>(dyns: &'a [Dyn], strtab: &'b Strtab<'a>, count: usize) -> Vec<&'a str> {
    let mut needed = Vec::with_capacity(count);
    for dyn in dyns {
        if dyn.d_tag == DT_NEEDED {
            let lib = strtab.get(dyn.d_val as usize);
            needed.push(lib);
        }
    }
    needed
}

pub unsafe fn debug_print_dynamic(dynamic: &[Dyn]) {
    for dyn in dynamic {
        dyn.debug_print();
    }
}

/* TODO add these
/* Values of `d_un.d_val' in the DT_FLAGS entry.  */
#define DF_ORIGIN	0x00000001	/* Object may use DF_ORIGIN */
#define DF_SYMBOLIC	0x00000002	/* Symbol resolutions starts here */
#define DF_TEXTREL	0x00000004	/* Object contains text relocations */
#define DF_BIND_NOW	0x00000008	/* No lazy binding for this object */
#define DF_STATIC_TLS	0x00000010	/* Module uses the static TLS model */

/* State flags selectable in the `d_un.d_val' element of the DT_FLAGS_1
   entry in the dynamic section.  */
#define DF_1_NOW	0x00000001	/* Set RTLD_NOW for this object.  */
#define DF_1_GLOBAL	0x00000002	/* Set RTLD_GLOBAL for this object.  */
#define DF_1_GROUP	0x00000004	/* Set RTLD_GROUP for this object.  */
#define DF_1_NODELETE	0x00000008	/* Set RTLD_NODELETE for this object.*/
#define DF_1_LOADFLTR	0x00000010	/* Trigger filtee loading at runtime.*/
#define DF_1_INITFIRST	0x00000020	/* Set RTLD_INITFIRST for this object*/
#define DF_1_NOOPEN	0x00000040	/* Set RTLD_NOOPEN for this object.  */
#define DF_1_ORIGIN	0x00000080	/* $ORIGIN must be handled.  */
#define DF_1_DIRECT	0x00000100	/* Direct binding enabled.  */
#define DF_1_TRANS	0x00000200
#define DF_1_INTERPOSE	0x00000400	/* Object is used to interpose.  */
#define DF_1_NODEFLIB	0x00000800	/* Ignore default lib search path.  */
#define DF_1_NODUMP	0x00001000	/* Object can't be dldump'ed.  */
#define DF_1_CONFALT	0x00002000	/* Configuration alternative created.*/
#define DF_1_ENDFILTEE	0x00004000	/* Filtee terminates filters search. */
#define	DF_1_DISPRELDNE	0x00008000	/* Disp reloc applied at build time. */
#define	DF_1_DISPRELPND	0x00010000	/* Disp reloc applied at run-time.  */
#define	DF_1_NODIRECT	0x00020000	/* Object has no-direct binding. */
#define	DF_1_IGNMULDEF	0x00040000
#define	DF_1_NOKSYMS	0x00080000
#define	DF_1_NOHDR	0x00100000
#define	DF_1_EDITED	0x00200000	/* Object is modified after built.  */
#define	DF_1_NORELOC	0x00400000
#define	DF_1_SYMINTPOSE	0x00800000	/* Object has individual interposers.  */
#define	DF_1_GLOBAUDIT	0x01000000	/* Global auditing required.  */
#define	DF_1_SINGLETON	0x02000000	/* Singleton symbols are used.  */

*/
