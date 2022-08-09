pub mod arith;
flat_mod!(sum);

use std::{mem::MaybeUninit, ops::{Deref, DerefMut}, fmt::Debug};
use blaze_rs::{prelude::*, buffer::KernelPointer};
use crate::{Real, include_prog};
use self::arith::*;

#[blaze(VectorProgram<T: Real>)]
#[link = include_prog::<T>(include_str!("../opencl/vec.cl"))]
pub extern "C" {
    fn add (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
    fn sub (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
    fn scal (n: usize, alpha: T, rhs: *const T, out: *mut MaybeUninit<T>);
    fn scal_down (n: usize, lhs: *const T, alpha: T, out: *mut MaybeUninit<T>);
    fn scal_down_inv (n: usize, alpha: T, rhs: *const T, out: *mut MaybeUninit<T>);
    #[link_name = "sum"]
    fn vec_sum (n: usize, lhs: *const T, out: *mut MaybeUninit<T>);
    fn sum_cpu (n: usize, lhs: *const T, out: *mut MaybeUninit<T>);
    fn sum_atomic (n: usize, lhs: *const u32, out: *mut MaybeUninit<u32>);
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

    #[inline(always)]
    pub const fn from_buffer (inner: Buffer<T>) -> Self {
        Self { inner }
    }

    #[inline(always)]
    pub fn into_buffer (self) -> Buffer<T> {
        self.inner
    }
}

// ADDITION
impl<T: Real> Vector<T> {
    #[inline(always)]
    pub fn add<RHS: Deref<Target = Self>> (&self, other: RHS, wait: impl Into<WaitList>) -> Result<Addition<T, &'_ Self, RHS>> {
        Self::add_by_deref(self, other, wait)
    }

    #[inline(always)]
    pub unsafe fn add_unchecked<RHS: Deref<Target = Self>> (&self, other: RHS, wait: impl Into<WaitList>) -> Result<Addition<T, &'_ Self, RHS>> {
        Self::add_unchecked_by_deref(self, other, wait)
    }

    #[inline]
    pub fn add_by_deref<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (this: LHS, other: RHS, wait: impl Into<WaitList>) -> Result<Addition<T, LHS, RHS>> {
        let len = this.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorType::InvalidBufferSize, "Vectors must be of the same length"));
        }

        unsafe {
            Addition::new_custom(this, other, len, wait)
        }
    }

    #[inline(always)]
    pub unsafe fn add_unchecked_by_deref<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (this: LHS, other: RHS, wait: impl Into<WaitList>) -> Result<Addition<T, LHS, RHS>> {
        let len = this.len()?;
        Addition::new_custom(this, other, len, wait)
    }
}

// SUBTRACTION
impl<T: Real> Vector<T> {
    #[inline(always)]
    pub fn sub<RHS: Deref<Target = Self>> (&self, other: RHS, wait: impl Into<WaitList>) -> Result<Subtraction<T, &'_ Self, RHS>> {
        Self::sub_by_deref(self, other, wait)
    }

    #[inline(always)]
    pub unsafe fn sub_unchecked<RHS: Deref<Target = Self>> (&self, other: RHS, wait: impl Into<WaitList>) -> Result<Subtraction<T, &'_ Self, RHS>> {
        Self::sub_unchecked_by_deref(self, other, wait)
    }

    #[inline]
    pub fn sub_by_deref<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (this: LHS, other: RHS, wait: impl Into<WaitList>) -> Result<Subtraction<T, LHS, RHS>> {
        let len = this.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorType::InvalidBufferSize, "Vectors must be of the same length"));
        }

        unsafe{
            Subtraction::new_custom(this, other, len, wait)
        }
    }

    #[inline(always)]
    pub unsafe fn sub_unchecked_by_deref<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (this: LHS, other: RHS, wait: impl Into<WaitList>) -> Result<Subtraction<T, LHS, RHS>> {
        let len = this.len()?;
        Subtraction::new_custom(this, other, len, wait)
    }
}

// MULTIPLICATION
impl<T: Real> Vector<T> {
    #[inline(always)]
    pub fn mul (&self, alpha: T, wait: impl Into<WaitList>) -> Result<Scale<T, &'_ Self>> {
        Self::mul_by_deref(alpha, self, wait)
    }

    #[inline]
    pub fn mul_by_deref<RHS: Deref<Target = Self>> (alpha: T, this: RHS, wait: impl Into<WaitList>) -> Result<Scale<T, RHS>> {
        let len = this.len()?;
        unsafe{
            Scale::new_custom(alpha, this, len, wait)
        }
    }
}

// DIVISION
impl<T: Real> Vector<T> {
    #[inline(always)]
    pub fn div (&self, alpha: T, wait: impl Into<WaitList>) -> Result<Division<T, &'_ Self>> {
        Self::div_by_deref(self, alpha, wait)
    }

    #[inline]
    pub fn div_by_deref<LHS: Deref<Target = Self>> (this: LHS, alpha: T, wait: impl Into<WaitList>) -> Result<Division<T, LHS>> {
        let len = this.len()?;
        unsafe {
            Division::new_custom(this, alpha, len, wait)
        }
    }

    #[inline(always)]
    pub fn div_inv (&self, alpha: T, wait: impl Into<WaitList>) -> Result<InvDivision<T, &'_ Self>> {
        Self::div_inv_by_deref(alpha, self, wait)
    }

    #[inline]
    pub fn div_inv_by_deref<RHS: Deref<Target = Self>> (alpha: T, this: RHS, wait: impl Into<WaitList>) -> Result<InvDivision<T, RHS>> {
        let len = this.len()?;
        unsafe {
            InvDivision::new_custom(alpha, this, len, wait)
        }
    }
}

impl<T: Copy> Vector<MaybeUninit<T>> {
    #[inline(always)]
    pub unsafe fn assume_init (self) -> Vector<T> {
        Vector { inner: self.inner.assume_init() }
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

impl<T: Copy> Debug for Vector<T> where Buffer<T>: Debug {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
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