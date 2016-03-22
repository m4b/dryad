use std::ops::Index;
use std::slice;
use std::str;
use std::fmt;

pub struct Strtab<'mmap> {
    mmapped_strtab: &'mmap[u8],
}

#[inline(always)]
fn get_str<'a> (idx: usize, strtab: &'a[u8]) -> &str {
    let mut i = idx;
    let len = strtab.len();
    // hmmm, once exceptions are working correctly, maybe we should let this fail with i >= len?
    if i <= 0 || i >= len {
        return ""
    }
    let mut byte = strtab[i];
    while byte != 0 && i < strtab.len() {
        byte = strtab[i];
        i += 1;
    }
    if i > 0 { i -= 1; } // this isn't still quite right
    str::from_utf8(&strtab[idx..i]).unwrap()
}

impl<'mmap> Index<usize> for Strtab<'mmap> {
    type Output = str;

    fn index(&self, _index: usize) -> &Self::Output {
        get_str(_index, self.mmapped_strtab)
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

    /// Thanks to reem on #rust for this suggestion
    pub fn get (&self, idx: usize) -> &'mmap str {
        get_str(idx, self.mmapped_strtab)
    }

}
