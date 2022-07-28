flat_mod!(raw, complex, range);
use opencl_sys::{CL_MAP_READ, CL_MAP_WRITE};

use blaze_proc::docfg;
use crate::{prelude::{Context, RawKernel, Result, RawEvent}, event::WaitList, svm::Svm};

#[cfg(feature = "svm")]
use crate::svm::SvmPointer;
use self::rect::BufferRect2D;

pub mod rect;
pub mod flags;
pub mod events;

pub unsafe trait KernelPointer<T: Sync> {
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut WaitList, idx: u32) -> Result<()>;
    fn complete (&self, event: &RawEvent) -> Result<()>;
}

unsafe impl<T: Copy + Sync, C: Context> KernelPointer<T> for Buffer<T, C> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, _wait: &mut WaitList, idx: u32) -> Result<()> {
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
    unsafe fn set_arg (&self, kernel: &mut RawKernel, _wait: &mut WaitList, idx: u32) -> Result<()> {
        kernel.set_argument(idx, self.id_ref())
    }

    #[inline(always)]
    fn complete (&self, _event: &RawEvent) -> Result<()> {
        Ok(())
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Sync, P: SvmPointer<Type = T>> KernelPointer<T> for P where P::Context: 'static + Send + Clone {
    #[inline]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut WaitList, idx: u32) -> Result<()> {
        kernel.set_svm_argument(idx, self)?;

        if self.allocator().is_coarse() {
            let evt = self.allocator().unmap(self.as_ptr() as *mut _, WaitList::EMPTY)?;
            wait.push(evt)
        }

        Ok(())
    }

    #[inline]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        if self.allocator().is_coarse() {
            let alloc = Svm::clone(self.allocator());
            let size = core::mem::size_of::<T>() * self.len();
            let ptr = self.as_ptr() as *const T as usize;
            
            unsafe {
                let _ = alloc.map::<&RawEvent, {CL_MAP_READ | CL_MAP_WRITE}>(ptr as *mut _, size, event)?;
            }
        }

        Ok(())
    }
}