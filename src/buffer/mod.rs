use parking_lot::RawFairMutex;
use rscl_proc::docfg;

flat_mod!(raw, access);

mod sealed {
    pub trait Sealed {}
}

pub(crate) mod manager;
pub mod flags;
pub mod events;

use crate::{context::Context, core::{Kernel, Result}, event::WaitList, utils::{OwnedMutexGuard, OwnedMutex}};
use std::ffi::c_void;
use sealed::Sealed;

use self::manager::AccessManager;

pub unsafe trait ReadablePointer<T>: Sealed {
    fn get_ptr (&self) -> *mut c_void;
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>>;
}

unsafe impl<T: Copy + Unpin, C: Context> ReadablePointer<T> for Buffer<T, C> {
    #[inline(always)]
    fn get_ptr (&self) -> *mut c_void {
        self.as_ref().id()
    }

    #[inline]
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>> {
        let access = self.access_mananer().lock_owned();
        access.extend_to_read(wait);
        kernel.set_argument(idx, self.as_ref().id_ref())?;
        Ok(Some(access))
    }
}

unsafe impl<T: Copy + Unpin, C: Context> ReadablePointer<T> for ReadBuffer<T, C> {
    #[inline(always)]
    fn get_ptr (&self) -> *mut c_void {
        self.as_ref().id()
    }

    #[inline]
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>> {
        let access = self.access_mananer().lock_owned();
        access.extend_to_read(wait);
        kernel.set_argument(idx, self.as_ref().id_ref())?;
        Ok(Some(access))
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Copy + Unpin, C: Context> ReadablePointer<T> for crate::svm::SvmBox<T, C> {
    #[inline(always)]
    fn get_ptr (&self) -> *mut c_void {
        let rf : &T = ::core::ops::Deref::deref(self);
        rf as *const _ as *mut _
    }

    #[inline(always)]
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, _wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>> {
        kernel.set_svm_argument(idx, self)?;
        Ok(None)
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Copy + Unpin, C: Context> ReadablePointer<T> for crate::svm::SvmVec<T, C> {
    #[inline(always)]
    fn get_ptr (&self) -> *mut c_void {
        self.as_ptr() as *mut _
    }

    #[inline(always)]
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, _wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>> {
        kernel.set_svm_argument(idx, self)?;
        Ok(None)
    }
}

pub unsafe trait WriteablePointer<T>: Sealed {
    fn get_ptr (&mut self) -> *mut c_void;
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>>;
}

unsafe impl<T: Copy + Unpin, C: Context> WriteablePointer<T> for Buffer<T, C> {
    #[inline(always)]
    fn get_ptr (&mut self) -> *mut c_void {
        self.as_ref().id()
    }

    #[inline]
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>> {
        let access = self.access_mananer().lock_owned();
        access.extend_to_write(wait);
        kernel.set_argument(idx, self.as_ref().id_ref())?;
        Ok(Some(access))
    }
}

unsafe impl<T: Copy + Unpin, C: Context> WriteablePointer<T> for WriteBuffer<T, C> {
    #[inline(always)]
    fn get_ptr (&mut self) -> *mut c_void {
        self.as_ref().id()
    }

    #[inline]
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>> {
        let access = self.access_mananer().lock_owned();
        access.extend_to_write(wait);
        kernel.set_argument(idx, self.as_ref().id_ref())?;
        Ok(Some(access))
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Copy + Unpin, C: Context> WriteablePointer<T> for crate::svm::SvmBox<T, C> {
    #[inline(always)]
    fn get_ptr (&mut self) -> *mut c_void {
        let rf : &mut T = ::core::ops::DerefMut::deref_mut(self);
        rf as *mut _ as *mut _
    }

    #[inline(always)]
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, _wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>> {
        kernel.set_svm_argument(idx, self)?;
        Ok(None)
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Copy + Unpin, C: Context> WriteablePointer<T> for crate::svm::SvmVec<T, C> {
    #[inline(always)]
    fn get_ptr (&mut self) -> *mut c_void {
        self.as_mut_ptr().cast()
    }

    #[inline(always)]
    unsafe fn set_argument (&self, kernel: &mut Kernel, idx: u32, _wait: &mut WaitList) -> Result<Option<OwnedMutexGuard<RawFairMutex, AccessManager>>> {
        kernel.set_svm_argument(idx, self)?;
        Ok(None)
    }
}

impl<T: Copy + Unpin, C: Context> Sealed for Buffer<T, C> {}
impl<T: Copy + Unpin, C: Context> Sealed for ReadBuffer<T, C> {}
impl<T: Copy + Unpin, C: Context> Sealed for WriteBuffer<T, C> {}
#[docfg(feature = "svm")]
impl<T: Copy + Unpin, C: Context> Sealed for crate::svm::SvmBox<T, C> {}
#[docfg(feature = "svm")]
impl<T: Copy + Unpin, C: Context> Sealed for crate::svm::SvmVec<T, C> {}