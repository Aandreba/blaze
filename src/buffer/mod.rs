flat_mod!(raw, complex, range, map);

#[cfg(feature = "cl1_1")]
flat_mod!(slice);

use blaze_proc::docfg;
use crate::{prelude::{Context, RawKernel, Result, RawEvent}};

#[cfg(feature = "svm")]
use crate::svm::{Svm, SvmBox, SvmVec, SvmPointer};

pub mod rect;
pub mod flags;

pub unsafe trait KernelPointer<T: Sync> {
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut Vec<RawEvent>, idx: u32) -> Result<()>;
    fn complete (&self, event: &RawEvent) -> Result<()>;
}

unsafe impl<T: Copy + Sync, C: Context> KernelPointer<T> for Buffer<T, C> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, _wait: &mut Vec<RawEvent>, idx: u32) -> Result<()> {
        kernel.set_argument::<opencl_sys::cl_mem, _>(idx, self.id_ref())
    }

    #[inline(always)]
    fn complete (&self, _event: &RawEvent) -> Result<()> {
        Ok(())
    }
}

unsafe impl<T: Copy + Sync, C: Context> KernelPointer<T> for rect::RectBuffer2D<T, C> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, _wait: &mut Vec<RawEvent>, idx: u32) -> Result<()> {
        kernel.set_argument::<opencl_sys::cl_mem, _>(idx, self.id_ref())
    }

    #[inline(always)]
    fn complete (&self, _event: &RawEvent) -> Result<()> {
        Ok(())
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Sync, C: Context> KernelPointer<T> for SvmBox<[T], C> where C: 'static + Send + Clone {
    #[inline]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut Vec<RawEvent>, idx: u32) -> Result<()> {
        kernel.set_svm_argument::<T, Self>(idx, self)?;

        if Box::allocator(self).is_coarse() {
            let evt = Box::allocator(self).unmap(SvmPointer::<T>::as_ptr(self) as *mut _, None)?;
            wait.push(evt)
        }

        Ok(())
    }

    #[inline]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        if Box::allocator(self).is_coarse() {
            let alloc = Svm::clone(Box::allocator(self));
            let size = core::mem::size_of::<T>() * SvmPointer::<T>::len(self);
            let ptr = self.as_ptr() as *const T as usize;
            
            unsafe {
                let _ = alloc.map::<{opencl_sys::CL_MAP_READ | opencl_sys::CL_MAP_WRITE}>(
                    ptr as *mut _,
                    size,
                    Some(core::slice::from_ref(event))
                )?;
            }
        }

        Ok(())
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Sync, C: Context> KernelPointer<T> for SvmBox<T, C> where C: 'static + Send + Clone {
    #[inline]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut Vec<RawEvent>, idx: u32) -> Result<()> {
        kernel.set_svm_argument::<T, Self>(idx, self)?;

        if Box::allocator(self).is_coarse() {
            let evt = Box::allocator(self).unmap(SvmPointer::<T>::as_ptr(self) as *mut _, None)?;
            wait.push(evt)
        }

        Ok(())
    }

    #[inline]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        if Box::allocator(self).is_coarse() {
            let alloc = Svm::clone(Box::allocator(self));
            let size = core::mem::size_of::<T>();
            let ptr = self.as_ptr() as *const T as usize;
            
            unsafe {
                let _ = alloc.map::<{opencl_sys::CL_MAP_READ | opencl_sys::CL_MAP_WRITE}>(
                    ptr as *mut _,
                    size,
                    Some(core::slice::from_ref(event))
                )?;
            }
        }

        Ok(())
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Sync, C: Context> KernelPointer<T> for SvmVec<T, C> where C: 'static + Send + Clone {
    #[inline]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut Vec<RawEvent>, idx: u32) -> Result<()> {
        kernel.set_svm_argument::<T, Self>(idx, self)?;

        if Vec::allocator(self).is_coarse() {
            let evt = Vec::allocator(self).unmap(SvmPointer::<T>::as_ptr(self) as *mut _, None)?;
            wait.push(evt)
        }

        Ok(())
    }

    #[inline]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        if Vec::allocator(self).is_coarse() {
            let alloc = Svm::clone(Vec::allocator(self));
            let size = core::mem::size_of::<T>() * SvmPointer::<T>::len(self);
            let ptr = self.as_ptr() as *const T as usize;
            
            unsafe {
                let _ = alloc.map::<{opencl_sys::CL_MAP_READ | opencl_sys::CL_MAP_WRITE}>(
                    ptr as *mut _,
                    size,
                    Some(core::slice::from_ref(event))
                )?;
            }
        }

        Ok(())
    }
}

/*
unsafe impl<T: Sync, P: KernelPointer<T>> KernelPointer<MaybeUninit<T>> for P {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut Vec<RawEvent>, idx: u32) -> Result<()> {
        <Self as KernelPointer<T>>::set_arg(&self, kernel, wait, idx)
    }

    #[inline(always)]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        <Self as KernelPointer<T>>::complete(&self, event)
    }
}
*/

// Whenever [this](https://github.com/rust-lang/rust/issues/48869) is fixed, this will be the generic implementation

/*#[docfg(feature = "svm")]
unsafe impl<T: Sync, P: SvmPointer<T>> KernelPointer<T> for P where P::Context: 'static + Send + Clone {
    #[inline]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut Vec<RawEvent>, idx: u32) -> Result<()> {
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
            let size = core::mem::size_of::<T>() * self.len();
            
            unsafe {
                let _ = self.allocator().map::<&RawEvent, {CL_MAP_READ | CL_MAP_WRITE}>(self.as_ptr(), size, event)?;
            }
        }

        Ok(())
    }
}*/