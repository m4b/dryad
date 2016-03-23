use std::ops::Index;
use std::slice;
use std::str;
use std::fmt;

pub struct Strtab<'a> {
    bytes: &'a[u8],
}

#[inline(always)]
fn get_str<'a> (idx: usize, bytes: &'a[u8]) -> &str {
    let mut i = idx;
    let len = bytes.len();
    // hmmm, once exceptions are working correctly, maybe we should let this fail with i >= len?
    if i <= 0 || i >= len {
        return ""
    }
    let mut byte = bytes[i];
    while byte != 0 && i < bytes.len() {
        byte = bytes[i];
        i += 1;
    }
    if i > 0 { i -= 1; } // this isn't still quite right
    str::from_utf8(&bytes[idx..i]).unwrap()
}

impl<'a> Index<usize> for Strtab<'a> {
    type Output = str;

    fn index(&self, _index: usize) -> &Self::Output {
        get_str(_index, self.bytes)
    }
}

impl<'a> fmt::Debug for Strtab<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", str::from_utf8(&self.bytes))
    }
}

impl<'a> Strtab<'a> {
    
    pub fn new (bytes_ptr: *const u8, size: usize) -> Strtab<'a> {
        Strtab {
            bytes: unsafe { slice::from_raw_parts(bytes_ptr, size) },
        }
    }

    /// Thanks to reem on #rust for this suggestion
    pub fn get (&self, idx: usize) -> &'a str {
        get_str(idx, self.bytes)
    }

}
