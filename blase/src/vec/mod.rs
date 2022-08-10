pub mod arith;
pub mod sum;
pub mod dot;
pub mod cmp;

use std::{mem::MaybeUninit, ops::{Deref, DerefMut}, fmt::Debug};
use blaze_rs::{prelude::*, buffer::KernelPointer};
use crate::{Real};
use self::arith::*;
pub use program::VectorProgram;

pub mod program {
    use blaze_proc::blaze;
    use crate::{Real, include_prog, max_work_group_size};
    use std::mem::MaybeUninit;

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
}

/// Euclidian vector
#[repr(transparent)]
pub struct EucVec<T: Copy> {
    inner: Buffer<T>
}

impl<T: Copy> EucVec<T> {
    /// Creates a new vector
    pub fn new (v: &[T], alloc: bool) -> Result<Self> {
        let inner = Buffer::new(v, MemAccess::default(), alloc)?;
        Ok(Self { inner })
    }

    /// Creates a new uninitialized vector
    #[inline(always)]
    pub fn new_uninit (len: usize, alloc: bool) -> Result<EucVec<MaybeUninit<T>>> {
        let inner = Buffer::<T, _>::new_uninit(len, MemAccess::default(), alloc)?;
        Ok(EucVec { inner })
    }

    /// Turns a buffer into a vector. This is a zero-cost operation.
    #[inline(always)]
    pub const fn from_buffer (inner: Buffer<T>) -> Self {
        Self { inner }
    }

    /// Turns the vector into a buffer. This is a zero-cost operation.
    #[inline(always)]
    pub fn into_buffer (self) -> Buffer<T> {
        self.inner
    }
}

// ADDITION
impl<T: Real> EucVec<T> {
    /// Returns an events that resolves to the addition of `self` and `other`.
    #[inline(always)]
    pub fn add<RHS: Deref<Target = Self>> (&self, other: RHS, wait: impl Into<WaitList>) -> Result<Addition<T, &'_ Self, RHS>> {
        Self::add_by_deref(self, other, wait)
    }

    /// Returns an events that resolves to the addition of `self` and `other`, without checking their sizes.
    /// # Safety
    /// This function is only safe is the lengths of `self` and `other` are equal.
    #[inline(always)]
    pub unsafe fn add_unchecked<RHS: Deref<Target = Self>> (&self, other: RHS, wait: impl Into<WaitList>) -> Result<Addition<T, &'_ Self, RHS>> {
        Self::add_unchecked_by_deref(self, other, wait)
    }

    /// Returns an events that resolves to the addition of `this` and `other`.
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

    /// Returns an events that resolves to the addition of `this` and `other`, without checking their sizes.
    /// # Safety
    /// This function is only safe is the lengths of `this` and `other` are equal.
    #[inline(always)]
    pub unsafe fn add_unchecked_by_deref<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (this: LHS, other: RHS, wait: impl Into<WaitList>) -> Result<Addition<T, LHS, RHS>> {
        let len = this.len()?;
        Addition::new_custom(this, other, len, wait)
    }
}

// SUBTRACTION
impl<T: Real> EucVec<T> {
    /// Returns an events that resolves to the subtraction of `self` and `other`.
    #[inline(always)]
    pub fn sub<RHS: Deref<Target = Self>> (&self, other: RHS, wait: impl Into<WaitList>) -> Result<Subtraction<T, &'_ Self, RHS>> {
        Self::sub_by_deref(self, other, wait)
    }

    /// Returns an events that resolves to the subtraction of `self` and `other`, without checking their sizes.
    /// # Safety
    /// This function is only safe is the lengths of `self` and `other` are equal.
    #[inline(always)]
    pub unsafe fn sub_unchecked<RHS: Deref<Target = Self>> (&self, other: RHS, wait: impl Into<WaitList>) -> Result<Subtraction<T, &'_ Self, RHS>> {
        Self::sub_unchecked_by_deref(self, other, wait)
    }

    /// Returns an events that resolves to the addition of `this` and `other`.
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

    /// Returns an events that resolves to the addition of `this` and `other`, without checking their sizes.
    /// # Safety
    /// This function is only safe is the lengths of `this` and `other` are equal.
    #[inline(always)]
    pub unsafe fn sub_unchecked_by_deref<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (this: LHS, other: RHS, wait: impl Into<WaitList>) -> Result<Subtraction<T, LHS, RHS>> {
        let len = this.len()?;
        Subtraction::new_custom(this, other, len, wait)
    }
}

// MULTIPLICATION
impl<T: Real> EucVec<T> {
    /// Returns an events that resolves to the multiplication of `self` by `alpha`.
    #[inline(always)]
    pub fn mul (&self, alpha: T, wait: impl Into<WaitList>) -> Result<Scale<T, &'_ Self>> {
        Self::mul_by_deref(alpha, self, wait)
    }

    /// Returns an events that resolves to the multiplication of `this` by `alpha`.
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
    /// Returns an events that resolves to the division of `self` by `alpha`.
    #[inline(always)]
    pub fn div (&self, alpha: T, wait: impl Into<WaitList>) -> Result<Division<T, &'_ Self>> {
        Self::div_by_deref(self, alpha, wait)
    }

    /// Returns an events that resolves to the division of `this` by `alpha`.
    #[inline(always)]
    pub fn div_by_deref<LHS: Deref<Target = Self>> (this: LHS, alpha: T, wait: impl Into<WaitList>) -> Result<Division<T, LHS>> {
        let len = this.len()?;
        unsafe {
            Division::new_custom(this, alpha, len, wait)
        }
    }

    /// Returns an events that resolves to the division of `alpha` by `self`.
    #[inline(always)]
    pub fn div_inv (&self, alpha: T, wait: impl Into<WaitList>) -> Result<InvDivision<T, &'_ Self>> {
        Self::div_inv_by_deref(alpha, self, wait)
    }

    /// Returns an events that resolves to the division of `alpha` by `this`.
    #[inline(always)]
    pub fn div_inv_by_deref<RHS: Deref<Target = Self>> (alpha: T, this: RHS, wait: impl Into<WaitList>) -> Result<InvDivision<T, RHS>> {
        let len = this.len()?;
        unsafe {
            InvDivision::new_custom(alpha, this, len, wait)
        }
    }
}

impl<T: Copy> EucVec<MaybeUninit<T>> {
    /// Extracts the value from `EucVec<MaybeUninit<T>>` to `EucVec<T>`
    /// # Safety
    /// This function has the same safety as [`MaybeUninit`](std::mem::MaybeUninit)'s `assume_init`
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