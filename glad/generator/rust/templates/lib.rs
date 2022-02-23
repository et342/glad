#[allow(clippy::all)]
pub mod {{ spec.name }};

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

/// A cheap convenience wrapper around [`CStr`](std::ffi::CStr).
#[repr(transparent)]
pub struct FnName(CStr);

impl FnName {
    /// Returns a pointer to a NUL-terminated string.
    #[inline]
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }

    /// Returns a string without trailing NUL byte.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(self.0.to_bytes())
        }
    }

    #[inline]
    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }

    #[inline]
    unsafe fn from_bytes_with_nul_unchecked(bytes: &[u8]) -> &Self {
        std::mem::transmute(CStr::from_bytes_with_nul_unchecked(bytes))
    }
}

#[derive(Copy, Clone)]
struct FnPtr {
    ptr: *const c_void,
}

#[allow(dead_code)]
impl FnPtr {
    const fn new(ptr: *const c_void) -> FnPtr {
        FnPtr {
            ptr
        }
    }

    fn set_ptr(&mut self, ptr: *const c_void) {
        self.ptr = ptr;
    }

    fn is_loaded(self) -> bool {
        !self.ptr.is_null()
    }

    fn aliased(&mut self, other: &FnPtr) {
        if !self.is_loaded() && other.is_loaded() {
            *self = *other;
        }
    }
}

unsafe impl Sync for FnPtr {}
unsafe impl Send for FnPtr {}
