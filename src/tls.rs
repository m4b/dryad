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
