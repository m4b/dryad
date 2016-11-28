use libc;
use elf::dyn;
use image::SharedObject;
use std::ffi::CString;
use std::fmt;
use std::default::Default;
use std::clone::Clone;

#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum State {
    // This state value describes the mapping change taking place when
    // the `r_brk' address is called.
    RT_CONSISTENT = 0, // Mapping change is complete.
    RT_ADD = 1, // Beginning to add a new object.
    RT_DELETE = 2 // Beginning to remove an object mapping.
}

#[repr(C)]
#[derive(Copy)]
pub struct LinkMap {
    pub l_addr: usize,
    pub l_name: *const libc::c_char,
    pub l_ld: *const dyn::Dyn,
    pub l_next: *mut LinkMap,
    pub l_prev: *mut LinkMap,
}
impl Clone for LinkMap {
    fn clone(&self) -> Self { *self }
}
impl Default for LinkMap {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}
impl fmt::Debug for LinkMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {:x} {:?}",
               unsafe { CString::from_raw(self.l_name as *mut libc::c_char)},
               self.l_addr,
               self.l_ld,
               )
    }
}

impl LinkMap {

    pub fn new (addr: usize, path: &str, dynamic: &[dyn::Dyn]) -> LinkMap {
        let l_name = CString::new(path).unwrap().into_raw();

        LinkMap {
            l_addr: addr,
            l_name: l_name,
            l_ld: dynamic.as_ptr(),
            l_next: 0 as *mut LinkMap,
            l_prev: 0 as *mut LinkMap,
        }
    }

    pub fn from_so (so: &SharedObject) -> *mut LinkMap {
        let l_name = if let Some (load_path) = so.load_path.to_owned() {
            CString::new(load_path).unwrap().into_raw()
        } else {
            CString::new("").unwrap().into_raw()
        };

        Box::into_raw(Box::new(LinkMap {
            l_addr: so.load_bias,
            l_name: l_name as *const libc::c_char,
            l_ld: so.dynamic.as_ptr(),
            l_next: 0 as *mut LinkMap,
            l_prev: 0 as *mut LinkMap,
        }))
    }

    pub unsafe fn append (so: *mut LinkMap, mut l: *mut LinkMap) {
        while !(*l).l_next.is_null() {
            l = (*l).l_next;
        }
        (*l).l_next = so;
        (*so).l_prev = l;
    }

    pub unsafe fn cons (so: *mut LinkMap, l: *mut LinkMap) {
        (*l).l_prev = so;
        (*so).l_next = l;
    }

}

#[repr(C)]
#[derive(Copy, Debug)]
pub struct Debug {
    pub r_version: libc::c_int,
    pub r_map: *mut LinkMap,
    pub r_brk: usize,
    pub r_state: State,
    pub r_ldbase: usize,
}
impl Clone for Debug {
    fn clone(&self) -> Self { *self }
}
impl Default for Debug {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

impl Debug {

    /// WARNING: We must initialize after relocation, otherwise the `r_brk` function address will be invalid
    pub unsafe fn relocated_init (&mut self, base: usize) {
        self.r_ldbase = base;
        self.r_brk = &r_debug_state as *const _ as usize;
        self.r_state = State::RT_CONSISTENT;
        // i think gdb likes it when there is a first "null" value here... So it can skip it. But it's hard to say, not done debugging the debugger yet. As David says, this is my life: http://m.xkcd.com/1671/
        self.r_map = Box::into_raw(Box::new(LinkMap::default()));
    }

    pub unsafe fn update (&self, state: State) {
        _r_debug.r_state = state;
        r_debug_state ();
    }

    pub unsafe fn add_so (&mut self, so: &SharedObject) {
        let lm = LinkMap::from_so(so);
        // this is not documented, but the debugger requires we append, and not cons (contrary to what you would think), since C programmers are all about the speeds - after all, who wants a constant prepend when you can have a linear append?
        LinkMap::append(lm, self.r_map);
    }
}

unsafe impl Send for Debug {}
unsafe impl Sync for Debug {}

pub unsafe fn insert_r_debug<'a, 'b> (dynamic: &[dyn::Dyn]) {
    for dyn in dynamic {
        if dyn.d_tag as u64 == dyn::DT_DEBUG {
            *((dyn as *const _ as *mut usize).offset(1)) = &_r_debug as *const Debug as usize;
            break;
        }
    }
}

/// this is one of the accepted symbols in gdb's `solib_break_names` array, e.g.:
/// "r_debug_state",
/// "_r_debug_state",
/// "_dl_debug_state",
/// "rtld_db_dlactivity",
/// "__dl_rtld_db_dlactivity",
/// "_rtld_debug_state",
#[no_mangle]
pub unsafe extern fn r_debug_state () {
}

/// `gdb` looks for this exact name in the binary referenced in the debugee's `PT_INTERPRETER`
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut _r_debug: Debug = Debug {
    r_version: 1, // according to dl-debug.c in glibc: R_DEBUG_VERSION XXX
    r_map: 0 as *mut LinkMap,
    r_brk: 0,
    r_state: State::RT_CONSISTENT,
    r_ldbase: 0
};
