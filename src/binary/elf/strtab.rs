use std::ops::Index;
use std::slice;
use std::str;
use std::fmt;

pub struct Strtab<'mmap> {
    mmapped_strtab: &'mmap[u8],
}

impl<'mmap> Index<usize> for Strtab<'mmap> {
    type Output = str;

    fn index(&self, _index: usize) -> &Self::Output {
        let mut i = _index;
        let len = self.mmapped_strtab.len();
        // hmmm, once exceptions are working correctly, maybe we should let this fail with i >= len?
        if i <= 0 || i >= len {
            return ""
        }
        let mut byte = self.mmapped_strtab[i];
        while byte != 0 && i < self.mmapped_strtab.len() {
            byte = self.mmapped_strtab[i];
            i += 1;
        }
        if i > 0 { i -= 1; } // this isn't still quite right
        str::from_utf8(&self.mmapped_strtab[_index..i]).unwrap()
    }
}

impl<'mmap> fmt::Debug for Strtab<'mmap> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", str::from_utf8(&self.mmapped_strtab))
    }
}

impl<'mmap> Strtab<'mmap> {
    
    pub fn new (strtab_ptr: *const u8, size: usize) -> Strtab<'mmap> {
        Strtab {
            mmapped_strtab: unsafe { slice::from_raw_parts(strtab_ptr, size) },
        }
    }
}
