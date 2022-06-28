use std::{ffi::c_void, ptr::addr_of};
use opencl_sys::{cl_mem, clSetKernelArg, clSetKernelArgSVMPointer};
use super::{Kernel, NdKernelEvent};
use crate::{core::*, buffer::Buffer, context::Context, svm::{SvmPointer}};

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
        println!("{:b}", parent.reference_count()?);

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
    pub fn set_mem_buffer<T: Copy> (&mut self, idx: usize, buffer: &Buffer<T, C>) -> &mut Self {
        self.args[idx] = Some(ArgumentType::Buffer(buffer.inner));
        self
    }

    #[inline(always)]
    pub fn set_svm<P: SvmPointer> (&mut self, idx: usize, svm: &'a P) -> &mut Self {
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
    Buffer (cl_mem),
    Svm (*const C),
    Alloc (usize)
}

impl<C: Context> ArgumentType<C> {
    #[inline(always)]
    pub fn arg_size (&self) -> usize {
        match self {
            Self::Value (x) => x.len(),
            Self::Buffer (_) => core::mem::size_of::<cl_mem>(),
            Self::Svm(_) => unimplemented!(),
            Self::Alloc (x) => *x 
        }
    }

    #[inline(always)]
    pub fn arg_value (&self) -> *const c_void {
        match self {
            Self::Value (x) => x.as_ptr().cast(),
            Self::Buffer (x) => x as *const _ as *const _,
            Self::Alloc (_) => core::ptr::null(),
            Self::Svm(_) => unimplemented!(),
        }
    }

    #[inline(always)]
    pub unsafe fn set_argument (&self, idx: u32, kernel: &Kernel<C>) -> Result<()> {
        match self {
            Self::Svm (ptr) => {
                tri!(clSetKernelArgSVMPointer(kernel.inner, idx, ptr.cast()))
            },

            _ => {
                tri!(clSetKernelArg(kernel.inner, idx, self.arg_size(), self.arg_value()));
            }
        }
        
        Ok(())
    }
}