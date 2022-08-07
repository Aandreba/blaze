use std::{mem::MaybeUninit, ops::{Deref, DerefMut}};
use blaze_rs::{prelude::*, buffer::KernelPointer};
use crate::{include_prog, Real, work_group_size};

//flat_mod!(arith);

#[blaze(VectorArith<T: Real>)]
#[link = include_prog::<T>(include_str!("../opencl/vec.cl"))]
pub extern "C" {
    fn add (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
    fn sub (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
}

#[repr(transparent)]
pub struct Vector<T: Copy> {
    inner: Buffer<T>
}

impl<T: Copy> Vector<T> {
    pub fn new (v: &[T], alloc: bool) -> Result<Self> {
        let inner = Buffer::new(v, MemAccess::default(), alloc)?;
        Ok(Self { inner })
    }

    #[inline(always)]
    pub fn new_uninit (len: usize, alloc: bool) -> Result<Vector<MaybeUninit<T>>> {
        let inner = Buffer::<T, _>::new_uninit(len, MemAccess::default(), alloc)?;
        Ok(Vector { inner })
    }
}

impl<T: Real> Vector<T> {
    #[inline(always)]
    pub unsafe fn add_unchecked_in (&self, other: &Self, prog: &VectorArith<T>) -> Result<()> {
        todo!()
    }

    #[inline]
    unsafe fn inner_add<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (lhs: LHS, rhs: RHS, len: usize, prog: &VectorArith<T>) -> Result<()> {
        let mut result = Self::new_uninit(len, false)?;
        let inner = unsafe {
            prog.add(len, lhs, rhs, &mut result.inner, [work_group_size(len)], None, EMPTY)?
        };

        

        todo!()
    }
}

impl<T: Copy> Deref for Vector<T> {
    type Target = Buffer<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Copy> DerefMut for Vector<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

unsafe impl<T: Copy + Sync> KernelPointer<T> for Vector<T> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut WaitList, idx: u32) -> Result<()> {
        self.inner.set_arg(kernel, wait, idx)
    }

    #[inline(always)]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        self.inner.complete(event)
    }
}