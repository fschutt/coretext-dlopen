use core::fmt;
extern crate alloc;

fn main() {
    println!("{:?}", load_coretext_lib());
}

fn load_coretext_lib() -> Result<(), String> {
    
    let coretext = Library::load("/System/Library/Frameworks/CoreText.framework/CoreText")
        .map_err(|e| format!("Could not load CoreText: {}", e))?;

    println!("ok: coretext loaded!");
    Ok(())
}


use std::ffi::{CString, OsStr};
use std::os::raw;

extern { // syscalls
    fn dlopen(filename: *const raw::c_char, flags: raw::c_int) -> *mut raw::c_void;
    fn dlsym(handle: *mut raw::c_void, symbol: *const raw::c_char) -> *mut raw::c_void;
    fn dlclose(handle: *mut raw::c_void) -> raw::c_int;
    fn dlerror() -> *mut raw::c_char;
}

/// A platform-specific equivalent of the cross-platform `Library`.
pub struct Library {
    name: &'static str,
    ptr: *mut raw::c_void
}

unsafe impl Send for Library {}
unsafe impl Sync for Library {}

impl Library {

    /// Dynamically load an arbitrary library by its name (dlopen)
    pub fn load(name: &'static str) -> Result<Self, String> {

        use alloc::borrow::Cow;
        use std::ffi::{CString, CStr};

        const RTLD_NOW: raw::c_int = 2;

        let cow = CString::new(name.as_bytes()).map_err(|e| String::new())?;
        let ptr = unsafe { dlopen(cow.as_ptr(), RTLD_NOW) };

        if ptr.is_null() {
            let dlerr = unsafe { CStr::from_ptr(dlerror()) };
            Err(dlerr.to_str().ok().map(|s| s.to_string()).unwrap_or_default())
        } else {
            Ok(Self { name, ptr })
        }
    }

    pub fn get(&self, symbol: &str) -> Option<*mut raw::c_void> {

        use std::ffi::CString;

        let symbol_name_new = CString::new(symbol.as_bytes()).ok()?;
        let symbol_new = unsafe { dlsym(self.ptr, symbol_name_new.as_ptr()) };
        let error = unsafe { dlerror() };
        if error.is_null() {
            Some(symbol_new)
        } else {
            None
        }
    }
}

impl fmt::Debug for Library {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl fmt::Display for Library {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { dlclose(self.ptr) };
    }
}
