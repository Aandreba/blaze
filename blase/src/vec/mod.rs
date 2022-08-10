pub mod arith;
flat_mod!(sum, dot, cmp);

use std::{mem::MaybeUninit, ops::{Deref, DerefMut}, fmt::Debug};
use blaze_rs::{prelude::*, buffer::KernelPointer};
use crate::{Real, include_prog, max_work_group_size};
use self::arith::*;

#[blaze(VectorProgram<T: Real>)]
#[link = generate_vec_program::<T>()]
pub extern "C" {
    fn add (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
    fn sub (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
    fn scal (n: usize, alpha: T, rhs: *const T, out: *mut MaybeUninit<T>);
    fn scal_down (n: usize, lhs: *const T, alpha: T, out: *mut MaybeUninit<T>);
    fn scal_down_inv (n: usize, alpha: T, rhs: *const T, out: *mut MaybeUninit<T>);
    #[link_name = "cmp"]
    fn vec_cmp_eq (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<u32>);
    #[link_name = "Xasum"]
    fn xasum (n: i32, x: *const T, output: *mut MaybeUninit<T>);
    #[link_name = "XasumEpilogue"]
    fn xasum_epilogue (input: *const MaybeUninit<T>, asum: *mut MaybeUninit<T>);
    #[link_name = "Xdot"]
    fn xdot (n: i32, x: *const T, y: *const T, output: *mut MaybeUninit<T>);
}

/// Euclidian vector
#[repr(transparent)]
pub struct EucVec<T: Copy> {
    inner: Buffer<T>
}

impl<T: Copy> EucVec<T> {
    pub fn new (v: &[T], alloc: bool) -> Result<Self> {
        let inner = Buffer::new(v, MemAccess::default(), alloc)?;
        Ok(Self { inner })
    }

    #[inline(always)]
    pub fn new_uninit (len: usize, alloc: bool) -> Result<EucVec<MaybeUninit<T>>> {
        let inner = Buffer::<T, _>::new_uninit(len, MemAccess::default(), alloc)?;
        Ok(EucVec { inner })
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
impl<T: Real> EucVec<T> {
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
impl<T: Real> EucVec<T> {
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
impl<T: Real> EucVec<T> {
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
impl<T: Real> EucVec<T> {
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

impl<T: Copy> EucVec<MaybeUninit<T>> {
    #[inline(always)]
    pub unsafe fn assume_init (self) -> EucVec<T> {
        EucVec { inner: self.inner.assume_init() }
    }
}

impl<T: Copy> Deref for EucVec<T> {
    type Target = Buffer<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Copy> DerefMut for EucVec<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Copy> Debug for EucVec<T> where Buffer<T>: Debug {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

unsafe impl<T: Copy + Sync> KernelPointer<T> for EucVec<T> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut WaitList, idx: u32) -> Result<()> {
        self.inner.set_arg(kernel, wait, idx)
    }

    #[inline(always)]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        self.inner.complete(event)
    }
}

fn generate_vec_program<T: Real> () -> String {
    // https://downloads.ti.com/mctools/esd/docs/opencl/execution/kernels-workgroups-workitems.html

    format!(
        "
            {0}
            #define WGS1 {1}
            #define WGS2 {1}
            {2}
            {3}
        ",
        include_prog::<T>(include_str!("../opencl/vec.cl")),
        usize::max(max_work_group_size().get() / 2, 2),
        include_str!("../opencl/blast_sum.cl"),
        include_str!("../opencl/blast_dot.cl")
    )
}