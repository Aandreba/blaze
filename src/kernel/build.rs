use std::{ffi::c_void, ptr::addr_of, sync::Arc};
use opencl_sys::{clSetKernelArg};
use parking_lot::{FairMutex, ReentrantMutexGuard};
use super::{Kernel, NdKernelEvent};
use crate::{core::*, buffer::{RawBuffer, flags::MemAccess, Buffer, manager::AccessManager}, context::Context, event::RawEvent, utils::{OwnedMutexGuard, OwnedMutex}};

#[derive(Clone)]
pub struct Build<'a, C: Context, const N: usize> {
    pub(super) parent: &'a Kernel<C>,
    pub(super) global_work_dims: [usize; N],
    pub(super) local_work_dims: Option<[usize; N]>,
    pub(super) args: Box<[Option<ArgumentType<C>>]>
}

impl<'a, C: Context, const N: usize> Build<'a, C, N> {
    #[inline(always)]
    pub fn new (parent: &'a Kernel<C>, global_work_dims: [usize; N]) -> Result<Self> {
        let arg_count = parent.num_args()? as usize;
        let mut args = Box::new_uninit_slice(arg_count);
        
        for i in 0..arg_count {
            args[i].write(None);
        }

        Ok(Self {
            parent,
            global_work_dims,
            local_work_dims: None,
            args: unsafe { args.assume_init() }
        })
    }

    #[inline(always)]
    pub fn global_work_dims (&mut self, v: [usize; N]) -> &mut Self {
        self.global_work_dims = v;
        self
    }

    #[inline(always)]
    pub fn local_work_dims (&mut self, v: impl Into<Option<[usize; N]>>) -> &mut Self {
        self.local_work_dims = v.into();
        self
    }

    #[inline(always)]
    pub fn set_value<T: Copy> (&mut self, idx: usize, v: T) -> &mut Self {
        let mut bytes = Box::new_uninit_slice(core::mem::size_of::<T>());
        let ty;

        unsafe {
            core::ptr::copy_nonoverlapping(addr_of!(v).cast(), bytes.as_mut_ptr() as *mut u8, core::mem::size_of::<T>());
            ty = ArgumentType::Value(bytes.assume_init());
        }

        self.args[idx] = Some(ty);
        self
    }

    #[inline(always)]
    pub unsafe fn set_raw_buffer (&mut self, idx: usize, buffer: RawBuffer, access: MemAccess) -> &mut Self {
        let access = ArgumentBuffer::Raw(access);
        self.args[idx] = Some(ArgumentType::Buffer(buffer, access));
        self
    }

    #[inline(always)]
    pub fn set_buffer<T: Copy> (&mut self, idx: usize, buffer: &Buffer<T, C>, access: MemAccess) -> &mut Self {
        let raw = unsafe { buffer.raw().clone() };
        let access = ArgumentBuffer::from_regular(buffer, access);

        self.args[idx] = Some(ArgumentType::Buffer(raw, access));
        self
    }

    #[cfg(feature = "svm")]
    #[inline(always)]
    pub unsafe fn set_svm<P: crate::svm::SvmPointer> (&mut self, idx: usize, svm: &'a P) -> &mut Self {
        self.args[idx] = Some(ArgumentType::Svm(svm.as_ptr().cast()));
        self
    }

    #[inline(always)]
    pub fn set_alloc<T: Copy> (&mut self, idx: usize, len: usize) -> &mut Self {
        let bytes = len.checked_mul(core::mem::size_of::<T>()).unwrap();
        self.args[idx] = Some(ArgumentType::Alloc(bytes));
        self
    }

    #[inline(always)]
    pub fn build (&self) -> Result<NdKernelEvent> {
        NdKernelEvent::new(self)
    }
}

#[derive(Clone)]
pub(super) enum ArgumentType<C> {
    Value (Box<[u8]>),
    Buffer (RawBuffer, ArgumentBuffer),

    #[cfg(feature = "svm")]
    Svm (*const C),
    Alloc (usize),

    #[cfg(not(feature = "svm"))]
    #[doc(hidden)]
    #[allow(dead_code)]
    Phantom (core::marker::PhantomData<C>),
}

impl<C: Context> ArgumentType<C> {
    #[inline(always)]
    pub fn arg_size (&self) -> usize {
        match self {
            Self::Value (x) => x.len(),
            Self::Buffer (_, _) => core::mem::size_of::<RawBuffer>(),
            Self::Alloc (x) => *x,
            #[cfg(debug_assertions)]
            _ => unreachable!(),
            #[cfg(not(debug_assertions))]
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }

    #[inline(always)]
    pub fn arg_value (&self) -> *const c_void {
        match self {
            Self::Value (x) => x.as_ptr().cast(),
            Self::Buffer (x, _) => x as *const _ as *const _,
            Self::Alloc (_) => core::ptr::null(),
            #[cfg(debug_assertions)]
            _ => unreachable!(),
            #[cfg(not(debug_assertions))]
            _ => unsafe { core::hint::unreachable_unchecked() }
        }
    }

    #[inline(always)]
    pub fn apply_effects (v: &[Option<ArgumentType<C>>], evt: RawEvent) {
        let managers = v.into_iter().filter_map(|x| match x {
            Some(ArgumentType::Buffer(_, x @ ArgumentBuffer::Regular(_, _))) => Some(x),
            _ => None
        });

        let managers = ArgumentBuffer::managers(managers);
    }

    #[inline(always)]
    pub unsafe fn set_argument (&self, idx: u32, kernel: &Kernel<C>) -> Result<()> {
        match self {
            #[cfg(feature = "svm")]
            Self::Svm (ptr) => {
                tri!(opencl_sys::clSetKernelArgSVMPointer(kernel.inner, idx, ptr.cast()))
            },

            _ => {
                tri!(clSetKernelArg(kernel.inner, idx, self.arg_size(), self.arg_value()));
            }
        }
        
        Ok(())
    }
}

#[derive(Clone)]
pub(super) enum ArgumentBuffer {
    Raw (MemAccess),
    Regular (MemAccess, Arc<FairMutex<AccessManager>>),
    // Todo read & write
}

impl ArgumentBuffer {
    #[inline(always)]
    pub fn from_regular<T: Copy, C: Context> (regular: &Buffer<T, C>, access: MemAccess) -> Self {
        Self::Regular(access, regular.access_mananer())
    }

    #[inline(always)]
    pub fn managers (v: impl IntoIterator<Item = Arc<FairMutex<AccessManager>>>) -> Vec<OwnedMutexGuard<parking_lot::RawFairMutex, AccessManager>> {
        v.into_iter().map(|x| x.lock_owned()).collect::<Vec<_>>()
    }
}