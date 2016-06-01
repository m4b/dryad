use libc;

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
    pub fn new (){
        unimplemented!();
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
#[derive(Debug)]
struct Pointer {
    val: *mut libc::c_void,
    is_static: bool
}

#[repr(C)]
#[derive(Debug)]
struct Dtv {
    counter: usize,
    pointer: Pointer
}

pub const SIZEOF_DTV: usize = 0x10;

#[inline(always)]
unsafe fn get_dtv(tls_storage: *mut libc::c_void) -> *mut Dtv {
    let descr_dtv = tls_storage.offset(1) as *mut *mut Dtv;
    dbgc!(purple_bold: true, "tls", "get_dtv: {:?}", **descr_dtv);
    *descr_dtv
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DtvInfo {
    generation: u32,
    info: TlsInfo
}

#[inline(always)]
/// Installs the dtv in the tcbhead_t struct, which is the second element; we hack it for now by simply casting and setting the offset; see `sysdeps/x86_64/nptl/tls.h:128`
unsafe fn install_dtv(tls_storage: *mut libc::c_void, dtv: *mut Dtv) {
    let descr_dtv = tls_storage.offset(1) as *mut *mut Dtv;
    *descr_dtv = dtv.offset(1);
}

unsafe fn allocate_dtv (tls_storage: *mut libc::c_void) -> *mut libc::c_void {
    let dtv_len = 2 + DTV_SURPLUS;
    dbgc!(purple_bold: true, "tls", "allocate_dtv dtv_len: {:?}", dtv_len);
    let dtv = libc::calloc(dtv_len + 2, SIZEOF_DTV as libc::size_t) as *mut Dtv;
    dbgc!(purple_bold: true, "tls", "allocate_dtv calloc dtv: {:?}", dtv);
    if !dtv.is_null() {
        // hehe surries!
        (*dtv.offset(0)).counter = dtv_len;
        dbgc!(purple_bold: true, "tls", "allocate_dtv calloc dtv: {:?}", *dtv);
        install_dtv(tls_storage, dtv);
        dbgc!(purple_bold: true, "tls", "allocate_dtv calloc post install: {:?} dtv: {:?}", **(tls_storage.offset(1) as *mut *mut Dtv), *dtv);
        tls_storage
    } else {
        dbgc!(purple_bold: true, "tls", "allocate_dtv dtv is NULL");
        ::std::ptr::null_mut::<libc::c_void>()
    }
}

pub unsafe fn _dl_allocate_tls_storage () -> *mut libc::c_void {
    let mut result = libc::memalign(TLS_STATIC_SIZE, TLS_STATIC_ALIGN);
    dbgc!(purple_bold: true, "tls", "_dl_allocate_tls_storage result: {:?}", result);
    let allocated = result; // to be used by free in case fails, unimplemented
    result = result.offset(TLS_STATIC_SIZE as isize - TLS_TCB_SIZE as isize);
    dbgc!(purple_bold: true, "tls", "_dl_allocate_tls_storage post result: {:?}", result);
    memset(result as *mut u8, 0x0, TLS_TCB_SIZE);
    dbgc!(purple_bold: true, "tls", "_dl_allocate_tls_storage memset result: {:?}", result);
    allocate_dtv(result)
}

// TODO: finish
// dl-tls.c:448 _dl_allocate_tls_init (void *result)
// TODO: replace max_dtv_idx/modid and tls_clients with mut self
pub unsafe fn allocate_tls (max_dtv_idx: u32, tls_clients: Vec<DtvInfo>) -> *mut libc::c_void {
    let tls_storage = _dl_allocate_tls_storage ();
    if tls_storage.is_null() {
        panic!("Error: tls storage failed to allocate");
    }
    let dtv = get_dtv(tls_storage);
    // TODO: check if current dtv is large enough
    // do this by examining the len (which is 14 + whatever modules we loaded) and checking if it's smaller than our dtv_idx counter (basically last modid)
    // need to write a resize_dtv routine of course, too
    // and then call install_dtv
    let total = 0;
    let maxgen = 0;
    loop {
        let cnt = if total == 0 { 1 } else { 0 };
        for cnt in cnt..2 {
            if total + cnt > max_dtv_idx {
                break;
            }
//            maxgen = max(maxgen, 
        }
        // TODO: remove after fixed
        break;
    }
    unimplemented!();
//    tls_storage
}

// TODO: finish
// dl-tls.c:137 _dl_determine_tlsoffset (void)
unsafe fn determine_offset(clients: &mut[DtvInfo]) {
    unimplemented!();
}

// TODO: finish
// sysdeps/x86_64/nptl/tls.h:148 TLS_INIT_TP(thrdescr)
unsafe fn tls_init_tp (tcbp: *mut libc::c_void) {
    unimplemented!();
}

// for special tls just for local process i believe, so it can access errno, etc.
// TODO: rtld.c:572 init_tls (void)
// 1. allocates the necessary tcbp
// 2. allocates the slot info list
// 3. sets the appropriate data into the slot info list (determine_offset)
// 4. and installs it via syscall tls_init_tp to the main thread
unsafe fn init_tls() {
    unimplemented!();
}

// final process
// 1. init_tls in main thread = dryad
// 2. load, relocate, etc.
// 3. allocate_tls_init (tcbp)
// 4. call TLS_INIT_TP

pub const TLS_STATIC_SIZE: libc::size_t = 0x1000;
pub const TLS_STATIC_ALIGN: libc::size_t = 0x40;
pub const TLS_STATIC_ALIGN_MASK: libc::size_t = !(TLS_STATIC_ALIGN - 1);
// sizeof (struct pthread)
pub const TLS_TCB_SIZE: libc::size_t = 0x900;
// sysdeps/generic/ldsodefs.h:402
pub const DTV_SURPLUS: libc::size_t = 14;
// sysdeps/generic/ldsodefs.h:399
pub const TLS_SLOTINFO_SURPLUS: libc::size_t = 62;

/// https://en.wikipedia.org/wiki/Lachesis_(mythology)
pub struct Lachesis {
    pub clients: Vec<DtvInfo>
}
