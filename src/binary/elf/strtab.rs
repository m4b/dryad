use std::ops::Index;
use std::slice;
use std::str;
use std::fmt;

pub struct Strtab<'a> {
    mmapped_strtab: &'a[u8],
}

impl<'b> Index<usize> for Strtab<'b> {
    type Output = str;

    fn index<'a>(&'a self, _index: usize) -> &'a Self::Output {
        let mut i = _index;
        let mut byte = 1; // this is a total hack
        while byte != 0 && i < self.mmapped_strtab.len() {
            byte = self.mmapped_strtab[i];
            i += 1;
        }
        str::from_utf8(&self.mmapped_strtab[_index..i]).unwrap()
    }
}

impl<'a> fmt::Debug for Strtab<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", str::from_utf8(&self.mmapped_strtab))
    }
}

impl<'a> Strtab<'a> {
    
    pub fn new<'b> (strtab_ptr: *const u8, size: usize) -> Strtab<'b> {
        Strtab {
            mmapped_strtab: unsafe { slice::from_raw_parts(strtab_ptr, size) },
        }
    }
}
