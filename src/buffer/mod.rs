flat_mod!(raw, access);

mod sealed {
    pub trait Sealed {}
}

pub(crate) mod manager;
pub mod flags;
pub mod events;

use crate::context::Context;
use std::ffi::c_void;
use sealed::Sealed;

pub unsafe trait ReadablePointer<T>: Sealed {
    fn get_ptr (&self) -> *mut c_void;
}

unsafe impl<T: Copy + Unpin, C: Context> ReadablePointer<T> for Buffer<T, C> {
    #[inline(always)]
    fn get_ptr (&self) -> *mut c_void {
        self.as_ref().id()
    }
}

unsafe impl<T: Copy + Unpin, C: Context> ReadablePointer<T> for ReadBuffer<T, C> {
    #[inline(always)]
    fn get_ptr (&self) -> *mut c_void {
        self.as_ref().id()
    }
}

#[cfg(feature = "svm")]
unsafe impl<T: Copy + Unpin, C: Context> ReadablePointer<T> for SvmBox<T, C> {
    #[inline(always)]
    fn get_ptr (&self) -> *mut c_void {
        let rf : &T = ::core::ops::Deref(self);
        rf as *const _ as *mut _
    }
}

#[cfg(feature = "svm")]
unsafe impl<T: Copy + Unpin, C: Context> ReadablePointer<T> for SvmVec<T, C> {
    #[inline(always)]
    fn get_ptr (&self) -> *mut c_void {
        self.as_ptr() as *mut _
    }
}

pub unsafe trait WriteablePointer<T>: Sealed {
    fn get_ptr (&mut self) -> *mut c_void;
}

unsafe impl<T: Copy + Unpin, C: Context> WriteablePointer<T> for Buffer<T, C> {
    #[inline(always)]
    fn get_ptr (&mut self) -> *mut c_void {
        self.as_ref().id()
    }
}

unsafe impl<T: Copy + Unpin, C: Context> WriteablePointer<T> for WriteBuffer<T, C> {
    #[inline(always)]
    fn get_ptr (&mut self) -> *mut c_void {
        self.as_ref().id()
    }
}

#[cfg(feature = "svm")]
unsafe impl<T: Copy + Unpin, C: Context> WriteablePointer<T> for SvmBox<T, C> {
    #[inline(always)]
    fn get_ptr (&mut self) -> *mut c_void {
        let rf : &mut T = ::core::ops::DerefMut(self);
        rf as *mut _ as *mut _
    }
}

#[cfg(feature = "svm")]
unsafe impl<T: Copy + Unpin, C: Context> WriteablePointer<T> for SvmVec<T, C> {
    #[inline(always)]
    fn get_ptr (&mut self) -> *mut c_void {
        self.as_mut_ptr().cast()
    }
}

impl<T: Copy + Unpin, C: Context> Sealed for Buffer<T, C> {}
impl<T: Copy + Unpin, C: Context> Sealed for ReadBuffer<T, C> {}
impl<T: Copy + Unpin, C: Context> Sealed for WriteBuffer<T, C> {}
#[cfg(feature = "svm")]
impl<T: Copy + Unpin, C: Context> Sealed for crate::svm::SvmBox<T> {}
#[cfg(feature = "svm")]
impl<T: Copy + Unpin, C: Context> Sealed for crate::svm::SvmVec<T> {}