flat_mod!(raw, access);
pub mod flags;
pub mod events;
pub(crate) mod manager;

use std::ffi::c_void;
use sealed::Sealed;

mod sealed {
    pub trait Sealed {}
}

pub trait ReadablePointer<T>: Sealed {
    unsafe fn get_ptr (&self) -> *mut c_void;
}

