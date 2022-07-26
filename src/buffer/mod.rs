flat_mod!(raw, complex, range);
use opencl_sys::{CL_MAP_READ, CL_MAP_WRITE};

use rscl_proc::docfg;
use crate::{prelude::{Context, Kernel, Result, RawEvent}, event::WaitList};

#[cfg(feature = "svm")]
use crate::svm::SvmPointer;

use self::rect::BufferRect2D;

pub mod rect;
pub mod flags;
pub mod events;

pub unsafe trait KernelPointer<T> {
    unsafe fn set_arg (&self, kernel: &mut Kernel, wait: &mut WaitList, idx: u32) -> Result<()>;
    fn complete (&self, event: &RawEvent) -> Result<()>;
}

unsafe impl<T: Copy + Sync, C: Context> KernelPointer<T> for Buffer<T, C> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut Kernel, _wait: &mut WaitList, idx: u32) -> Result<()> {
        kernel.set_argument(idx, self.id_ref())
    }

    #[inline(always)]
    fn complete (&self, _event: &RawEvent) -> Result<()> {
        Ok(())
    }
}

#[docfg(feature = "cl1_1")]
unsafe impl<T: Copy + Sync, C: Context> KernelPointer<T> for BufferRect2D<T, C> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut Kernel, _wait: &mut WaitList, idx: u32) -> Result<()> {
        kernel.set_argument(idx, self.id_ref())
    }

    #[inline(always)]
    fn complete (&self, _event: &RawEvent) -> Result<()> {
        Ok(())
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Sync, C: Context> KernelPointer<T> for crate::svm::SvmBox<T, C> where C: 'static + Send + Clone {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut Kernel, wait: &mut WaitList, idx: u32) -> Result<()> {
        kernel.set_svm_argument(idx, self)?;

        if self.allocator().is_coarse() {
            let evt = self.allocator().unmap(self.as_ptr() as *mut _, WaitList::EMPTY)?;
            wait.push(evt)
        }

        Ok(())
    }

    #[inline(always)]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        if self.allocator().is_coarse() {
            let alloc = self.allocator().clone();
            let ptr = self.as_ptr() as usize;
            
            event.on_complete(move |_, _| unsafe {
                alloc.map::<{CL_MAP_READ | CL_MAP_WRITE}>(ptr as *mut _, core::mem::size_of::<T>()).unwrap();
            })?;
        }

        Ok(())
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Sync, C: Context> KernelPointer<T> for crate::svm::SvmBox<[T], C> where C: 'static + Send + Clone {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut Kernel, wait: &mut WaitList, idx: u32) -> Result<()> {
        kernel.set_svm_argument(idx, self)?;

        if self.allocator().is_coarse() {
            let evt = self.allocator().unmap(self.as_ptr() as *mut _, WaitList::EMPTY)?;
            wait.push(evt)
        }

        Ok(())
    }

    #[inline(always)]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        if self.allocator().is_coarse() {
            let alloc = self.allocator().clone();
            let size = core::mem::size_of::<T>() * self.len();
            let ptr = self.as_ptr() as *const T as usize;
            
            event.on_complete(move |_, _| unsafe {
                alloc.map::<{CL_MAP_READ | CL_MAP_WRITE}>(ptr as *mut _, size).unwrap();
            })?;
        }

        Ok(())
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Sync, C: Context> KernelPointer<T> for crate::svm::SvmVec<T, C> where C: 'static + Send + Clone {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut Kernel, wait: &mut WaitList, idx: u32) -> Result<()> {
        kernel.set_svm_argument(idx, self)?;

        if self.allocator().is_coarse() {
            let evt = self.allocator().unmap(self.as_ptr() as *mut _, WaitList::EMPTY)?;
            wait.push(evt)
        }

        Ok(())
    }

    #[inline(always)]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        if self.allocator().is_coarse() {
            let alloc = self.allocator().clone();
            let size = core::mem::size_of::<T>() * self.len();
            let ptr = self.as_ptr() as *const T as usize;
            
            event.on_complete(move |_, _| unsafe {
                alloc.map::<{CL_MAP_READ | CL_MAP_WRITE}>(ptr as *mut _, size).unwrap();
            })?;
        }

        Ok(())
    }
}