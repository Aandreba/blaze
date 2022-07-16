flat_mod!(raw, complex, range);

#[cfg(feature = "cl1_1")]
pub use rect::BufferRect2D;
use rscl_proc::docfg;
use crate::{prelude::{Context, Kernel, Result}};

#[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
#[cfg(feature = "cl1_1")]
pub mod rect;
pub mod flags;
pub mod events;

pub unsafe trait KernelPointer<T> {
    unsafe fn set_arg (&self, kernel: &mut Kernel, idx: u32) -> Result<()>;
}

unsafe impl<T: Copy + Send, C: Context> KernelPointer<T> for Buffer<T, C> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut Kernel, idx: u32) -> Result<()> {
        kernel.set_argument(idx, self.id_ref())
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Send, C: Context> KernelPointer<T> for crate::svm::SvmBox<T, C> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut Kernel, idx: u32) -> Result<()> {
        kernel.set_svm_argument(idx, self)
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Send, C: Context> KernelPointer<T> for crate::svm::SvmBox<[T], C> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut Kernel, idx: u32) -> Result<()> {
        kernel.set_svm_argument(idx, self)
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Send, C: Context> KernelPointer<T> for crate::svm::SvmVec<T, C> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut Kernel, idx: u32) -> Result<()> {
        kernel.set_svm_argument(idx, self)
    }
}