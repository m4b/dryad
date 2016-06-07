use libc;
use std::cmp;
use binary::elf::program_header;

#[allow(non_camel_case_types)]
pub type size_t = ::std::os::raw::c_ulong;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Struct___locale_map { }

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct Struct___locale_struct {
    pub cat: [*const Struct___locale_map; 6usize],
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct tls_module {
    pub next: *mut tls_module,
    pub image: *mut ::std::os::raw::c_void,
    pub len: size_t,
    pub size: size_t,
    pub align: size_t,
    pub offset: size_t,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct __libc {
    pub can_do_threads: ::std::os::raw::c_int,
    pub threaded: ::std::os::raw::c_int,
    pub secure: ::std::os::raw::c_int,
    pub threads_minus_1: ::std::os::raw::c_int,
    pub auxv: *mut size_t,
    pub tls_head: *mut tls_module,
    pub tls_size: size_t,
    pub tls_align: size_t,
    pub tls_cnt: size_t,
    pub page_size: size_t,
    pub global_locale: Struct___locale_struct,
}

extern {
    pub static mut __libc: __libc;
    /// TLS init function which needs a pointer to aux vector indexed by AT_<TYPE> that musl likes
    pub fn __init_tls(aux: *const u64);
    static builtin_tls: *const u64;
}

// example of tls init structure for libc
/*
l_tls_initimage = 0x7ffff7bb2788,
l_tls_initimage_size = 16,
l_tls_blocksize = 120,
l_tls_align = 8,
l_tls_firstbyte_offset = 0,
l_tls_offset = 128,
l_tls_modid = 2,
l_tls_dtor_count = 0,
*/

#[derive(Debug, Clone, Copy)]
pub struct TlsInfo {
    pub blocksize: usize, // actual memory size ph_memsz
    pub align: usize,
    pub offset: isize,
    pub modid: u32,
    pub firstbyte_offset: usize,
    pub image: usize,
    pub image_size: usize,
}

impl TlsInfo {
    pub fn new (modid: u32, bias: usize, phdr: &program_header::ProgramHeader) -> TlsInfo {
        let blocksize = phdr.p_memsz as usize;
        let align = phdr.p_align as usize;
        let firstbyte_offset = if phdr.p_align == 0 { phdr.p_align } else { phdr.p_vaddr & (phdr.p_align - 1) } as usize;
        let image_size = phdr.p_filesz as usize;
        let image = phdr.p_vaddr as usize + bias;
        TlsInfo { blocksize: blocksize, align: align, offset: 0, modid: modid, firstbyte_offset: firstbyte_offset, image: image, image_size: image_size }
    }
}

/// Memset yo!
unsafe fn memset(ptr: *mut u8, byte: u8, size: usize) {
    let mut i = 0;
    while i < size {
        *ptr.offset(i as isize) = byte;
        i += 1;
    }
}

// glibc api
// always remember, for x86_64: TLS_TCB_AT_TP
// the TCB follows the TLS blocks

#[repr(C)]
#[derive(Debug, Clone)]
struct DtvHead {
    counter: usize,
    _padding: [u8; 8],
}

// this is a union :/, either counter or pointer
// the counter is only used for the first element in the dtv (DtvHead), because C programmers are dicks
#[repr(C)]
#[derive(Debug, Clone)]
struct Dtv {
    val: *mut libc::c_void,
    is_static: bool
}

pub const SIZEOF_DTV: usize = 0x10;

/// Difference between tls storage and dtv is 0x16f0
#[inline(always)]
unsafe fn get_dtv(tls_storage: *mut libc::c_void) -> *mut Dtv {
    let descr_dtv = tls_storage.offset(8) as *mut *mut Dtv;
    *descr_dtv
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SlotInfo {
    generation: u32,
    info: TlsInfo
}

//#[inline(always)]
/// Installs the dtv in the tcbhead_t struct, which is the second element (8 bytes out from the pointer); we hack it for now by simply casting and setting the offset; see `sysdeps/x86_64/nptl/tls.h:128`
unsafe fn install_dtv(tls_storage: *mut libc::c_void, dtv_head: *mut DtvHead) {
    let descr_dtv = tls_storage.offset(8) as *mut *mut Dtv;
    let dtv = dtv_head.offset(1) as *mut Dtv;
    *descr_dtv = dtv;
}

/// dl-tls.c:313 allocate_dtv (void *result)
unsafe fn allocate_dtv (max_dtv_idx: u32, tls_storage: *mut libc::c_void) -> *mut libc::c_void {
    let dtv_len = max_dtv_idx as usize + DTV_SURPLUS;
    dbgc!(purple_bold: true, "tls", "allocate_dtv dtv_len: {:?}", dtv_len);
    let dtv = libc::calloc(dtv_len + 2, SIZEOF_DTV as libc::size_t) as *mut DtvHead;
    dbgc!(purple_bold: true, "tls", "allocate_dtv calloc dtv: {:?}", dtv);
    if !dtv.is_null() {
        // hehe surries!
        (*dtv).counter = dtv_len;
        dbgc!(purple_bold: true, "tls", "allocate_dtv calloc dtv: {:?}", *dtv);
        install_dtv(tls_storage, dtv);
        dbgc!(purple_bold: true, "tls", "allocate_dtv calloc post install: {:?} dtv: {:?}", **(tls_storage.offset(8) as *mut *mut Dtv), *dtv);
        tls_storage
    } else {
        dbgc!(purple_bold: true, "tls", "allocate_dtv dtv is NULL");
        ::std::ptr::null_mut::<libc::c_void>()
    }
}

pub unsafe fn _dl_allocate_tls_storage (max_dtv_idx: u32, static_align: usize, static_size: usize, static_used: usize) -> *mut libc::c_void {

    // when DTV_AT_TP need to adjust static_size

    let mut result = libc::memalign(static_align, static_size);
    dbgc!(purple_bold: true, "tls", "_dl_allocate_tls_storage result: {:?}", result);
    let allocated = result; // to be used by free in case fails, unimplemented
    result = result.offset(static_size as isize - TLS_TCB_SIZE as isize);
    dbgc!(purple_bold: true, "tls", "_dl_allocate_tls_storage post result: {:?}", result);
    memset(result as *mut u8, 0x0, TLS_TCB_SIZE);
    dbgc!(purple_bold: true, "tls", "_dl_allocate_tls_storage memset result: {:?}", result);
    allocate_dtv(max_dtv_idx, result)
}

// TODO: finish
// dl-tls.c:448 _dl_allocate_tls_init (void *result)
// TODO: replace max_dtv_idx/modid and tls_clients with mut self
pub unsafe fn allocate_tls_init (max_dtv_idx: u32, tls_storage: *mut libc::c_void, modules: &[SlotInfo]) -> *mut libc::c_void {
    if tls_storage.is_null() {
        // TODO: actually don't think I can panic, because no TLS yet :/
        panic!("Error: tls storage failed to allocate");
    }
    let dtv = get_dtv(tls_storage);
    // TODO: check if current dtv is large enough
    // do this by examining the len (which is 14 + whatever modules we loaded) and checking if it's smaller than our dtv_idx counter (basically last modid)
    // need to write a resize_dtv routine of course, too
    // and then call install_dtv
    let mut total = 0;
    let mut maxgen = 0;
    dbgc!(purple_bold: true, "tls", "allocate_tls entering loop with modules: {:?}", modules);
    // TODO: fix this, broken w.r.t. glibc implementation because
    // slotinfo_list is a linked list of slotinfo_list, with len, next, and slotinfo[]; but we're just using the slotinfo[] here, because for simple test programs the linked list is always one element...
    loop {
        let cnt = if total == 0 { 1 } else { 0 };
        // TODO: this is broken here too with loop condition
        for cnt in 0..modules.len() {
            if total + cnt > max_dtv_idx as usize {
                dbgc!(purple_bold: true, "tls", "reached break total + cnt = {} > max_dtv_idx {}", total + cnt, max_dtv_idx);
                break;
            }
            dbgc!(purple_bold: true, "tls", "allocate_tls cnt: {:?} {:?}", cnt, TLS_DTV_UNALLOCATED);
            let info = modules[cnt].info;
            maxgen = cmp::max(maxgen, modules[cnt].generation);
            let mut dtv_ = dtv.offset(info.modid as isize) as *mut Dtv;
            *dtv_ = Dtv { val: TLS_DTV_UNALLOCATED, is_static: false};

            // cfg TLS_TCB_AT_TP
            let dest = (tls_storage.offset(-info.offset as isize)) as *mut u8;
            let size = info.blocksize - info.image_size;
            dbgc!(purple_bold: true, "tls", "memset + memcpy: {:?} with size: {} and info.image 0x{:x} info.image_size {}", dest, size, info.image, info.image_size);
            ::std::ptr::copy_nonoverlapping(dest, info.image as *mut u8, info.image_size);
//            libc::memcpy(dest, info.image, info.image_size);
            memset(dest, 0u8, size);
        }

        total += cnt;
        dbgc!(purple_bold: true, "tls", "allocate_tls new total: {:?}", total);
        if total >= max_dtv_idx as usize {
            break;
        }
    }

    dbgc!(purple_bold: true, "tls", "allocate_tls finished, setting dtv head to maxgen {} with tls storage: {:?}", maxgen, tls_storage);
    // i believe this is a bug in glibc
    // dtv[0].counter = maxgen
    // but dtv[-1] is the counter union ?
    *dtv = Dtv { val: maxgen as *mut libc::c_void, is_static: false};
    tls_storage
}

/// misc/sys/param.h
#[inline(always)]
fn roundup(x: usize, y: usize) -> usize {
    ((x + (y - 1)) / y) * y
}

/// Implements:
/// dl-tls.c:137 _dl_determine_tlsoffset (void)
fn determine_offset(static_align: &mut usize, static_used: &mut usize, static_size: &mut usize, modules: &mut[SlotInfo]) {
    let mut max_align = TLS_TCB_ALIGN;
    let mut freetop = 0;
    let mut freebottom = 0;
    assert!(modules.len() == 1);

    let mut offset = 0;

    for (i, slot_info) in modules.iter().enumerate() {
        let mut info = slot_info.info;
        let mut off;
        // TODO: refactor this to not mimic the insane C api
        let firstbyte = (-(info.firstbyte_offset as isize) & (info.align - 1) as isize) as usize;
        max_align = cmp::max(max_align, info.align);

        if (freebottom - freetop) >= info.blocksize {
            off = roundup ((freetop + info.blocksize) - firstbyte, info.align + firstbyte);
            if off <= freebottom {
                freetop = off;
                info.offset = off as isize;
                continue;
            }
        }

        off = roundup (offset + info.blocksize - firstbyte, info.align) + firstbyte;
        if off > offset + info.blocksize + (freebottom - freetop) {
            freetop = offset;
            freebottom = off - info.blocksize;
        }
        offset = off;
        info.offset = off as isize;
    }

    *static_used = offset;
    *static_size = roundup(offset + TLS_STATIC_SURPLUS, max_align) + TLS_TCB_SIZE;

    *static_align = max_align;

//    dbgc!(purple_bold: true, "tls", "determine_offset final static_used: {} static_size: {} static_align: {} freetop: {} freebottom: {}", static_used, static_size, static_align, freetop, freebottom);
}

// seeing segfault, notice shift of address when
// allocate_dtv calloc dtv: 0x1619640
//--- SIGSEGV {si_signo=SIGSEGV, si_code=SEGV_MAPERR, si_addr=0x161964f89} ---
/// Implements: sysdeps/x86_64/nptl/tls.h:148 TLS_INIT_TP(thrdescr)
/// sets up the `fs` thread pointer on x86_64
#[inline(always)]
pub unsafe fn tls_init_tp (tcbp: *mut libc::c_void) {
    let res = syscall!(ARCH_PRCTL, 0x1002, tcbp as usize);
}

// for special tls just for local process i believe, so it can access errno, etc.
// TODO: rtld.c:572 init_tls (void)
// 1. allocates the necessary tcbp
// 2. allocates the slot info list
// 3. sets the appropriate data into the slot info list (determine_offset)
// 4. and installs it via syscall tls_init_tp to the main thread
// 5. EXTRA: we immediately call allocate_tls_init for testing, which is normally called after the module information are all grabbed and accumulated into _rtld_global by the dynamic linker
pub unsafe fn init_tls(max_dtv_idx: u32, modules: &mut [SlotInfo]) {
    // this will be 64
    let nelem = max_dtv_idx + 1 + TLS_SLOTINFO_SURPLUS as u32;
    // TODO: this should probably be 0?
//    let mut static_align = 64;
//    let mut static_size = 4096;
//    let mut static_used = 120;

    let mut static_align = 0;
    let mut static_size = 0;
    let mut static_used = 0;

    dbgc!(purple_bold: true, "tls", "init_tls");
    determine_offset(&mut static_align, &mut static_used, &mut static_size, modules);

    dbgc!(purple_bold: true, "tls", "init_tls determine_offset {} {} {}", static_align, static_size, static_used);

    let tcbp = _dl_allocate_tls_storage(max_dtv_idx, static_align, static_size, static_used);
    dbgc!(purple_bold: true, "tls", "init_tls allocating tls storage {:?}", tcbp);
    // make the syscall
    //tls_init_tp(tcbp);
    dbgc!(purple_bold: true, "tls", "init_tls installed into thread");
    // glibc sets global tls_init_tp_called = true now and returns the tcbp for later use in allocate_tls_init(tcbp);
    allocate_tls_init(max_dtv_idx, tcbp, modules);
    dbgc!(purple_bold: true, "tls", "init_tls allocate_tls_init call finished");
    tls_init_tp(tcbp);
}

// final process
// 1. init_tls in main thread = dryad
// 2. load, relocate, etc.
// 3. allocate_tls_init (tcbp)

// sizeof (struct pthread)
pub const TLS_TCB_SIZE: libc::size_t = 0x900;
// sysdeps/generic/ldsodefs.h:402
pub const DTV_SURPLUS: libc::size_t = 14;
// sysdeps/generic/ldsodefs.h:399
pub const TLS_SLOTINFO_SURPLUS: libc::size_t = 62;

// this was the most miserable thing in the world to figure out
// __alignof__ (struct pthread)
// __alignof__ (struct pthread) says 8... but gdb shows this has 64
pub const TLS_TCB_ALIGN: libc::size_t = 64;

// "Non-shared code has no support for multiple namespaces."
// sysdeps/generic/ldsodefs.h:276
pub const DL_NNS: libc::size_t = 16;
// dl-tls.c:34
pub const TLS_STATIC_SURPLUS: libc::size_t = 64 + DL_NNS * 100;
pub const TLS_DTV_UNALLOCATED: *mut libc::c_void = 0xffffffffffffffff as *mut libc::c_void;

/// https://en.wikipedia.org/wiki/Lachesis_(mythology)
pub struct Lachesis {
    pub modules: Vec<SlotInfo>,
    pub current_modid: u32,
    debug: bool,
}

impl Lachesis {
    pub fn new(debug: bool) -> Lachesis {
        Lachesis { modules: Vec::with_capacity(3), current_modid: 0, debug: debug }
    }

    pub unsafe fn init_from_phdrs(bias: usize, phdrs: &[program_header::ProgramHeader])  {
        for phdr in phdrs {
            if phdr.p_type == program_header::PT_TLS {
                let tls = TlsInfo::new(0, bias, phdr);
                ::utils::write("in plt init tls\n");
                let mut static_align = 0;
                let mut static_size = 0;
                let mut static_used = 0;
                let mut modules = vec![SlotInfo { generation: 0, info: tls }];
                ::utils::write("mod\n");
                dbgc!(purple_bold: true, "tls", "init_tls");
                determine_offset(&mut static_align, &mut static_used, &mut static_size, &mut modules);
                ::utils::write("offset: ");
                ::utils::write("\nalign: ");
                ::utils::write_u64(static_align as u64, false);
                ::utils::write("\nsize: ");
                ::utils::write_u64(static_size as u64, false);
                ::utils::write("\nused: ");
                ::utils::write_u64(static_used as u64, false);
                ::utils::write("\n");
                let tcbp = _dl_allocate_tls_storage(1, static_align, static_size, static_used);
                dbgc!(purple_bold: true, "tls", "init_tls allocating tls storage {:?}", tcbp);
                ::utils::write("allocate tls storage: 0x");
                ::utils::write_u64(tcbp as u64, true);
                ::utils::write("\n");
                // make the syscall
                tls_init_tp(tcbp);
                ::utils::write("init tp\n");
                allocate_tls_init(1, tcbp, &modules);
                ::utils::write("allocate tls init\n");
                dbgc!(purple_bold: true, "tls", "init_tls allocate_tls_init call finished");

            }
        }
    }

    pub fn push_module(&mut self, soname: &str, bias: usize, phdr: &program_header::ProgramHeader) -> TlsInfo {
        let modid = { self.current_modid += 1; self.current_modid }; // increment, this will probably need to be atomic
        let tls = TlsInfo::new(modid, bias, phdr);
        dbgc!(purple_bold: self.debug, "lachesis", "PT_TLS in {} with {:?}", soname, tls);
        self.modules.push(SlotInfo { generation: 1, info: tls });
        tls
    }
}
