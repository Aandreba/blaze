use std::{ops::Deref, mem::MaybeUninit};
use blaze_rs::prelude::*;
use crate::{Real, utils::DerefCell, work_group_size};
use super::Vector;

pub struct Sum< T: Copy, LHS> {
    inner: super::VecSum<LHS, DerefCell<Buffer<MaybeUninit<T>>>, T>
}

pub struct SumWithSrc< T: Copy, LHS> {
    inner: super::VecSum<LHS, DerefCell<Buffer<MaybeUninit<T>>>, T>
}

impl<T: Real, LHS: Deref<Target = Vector<T>>> Sum<T, LHS> {
    #[inline]
    pub fn new_custom (lhs: LHS, wait: impl Into<WaitList>) -> Result<Self> {
        let len = lhs.len()?;
        let result = Buffer::new_uninit(1, MemAccess::READ_WRITE, false).map(DerefCell)?;
        let inner = unsafe {
            T::vec_program().vec_sum(len, lhs, result, [work_group_size(len)], None, wait)?
        };
        Ok(Self { inner })
    }

    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> SumWithSrc<T, LHS> {
        SumWithSrc { inner: self.inner }
    }
}

impl<T: Real, LHS: Deref<Target = Vector<T>>> Event for Sum<T, LHS> {
    type Output = T;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.inner.as_raw()
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let (_, out) = self.inner.consume(err)?;
        unsafe {
            let out : Buffer<T> = out.0.assume_init();
            let v = out.read(0..1, EMPTY)?.wait()?;
            Ok(*v.get_unchecked(0))
        }
    }
}

impl<T: Real, LHS: Deref<Target = Vector<T>>> Event for SumWithSrc<T, LHS> {
    type Output = (T, LHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.inner.as_raw()
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let (lhs, out) = self.inner.consume(err)?;
        unsafe {
            let out : Buffer<T> = out.0.assume_init();
            let v = out.read(0..1, EMPTY)?.wait()?;
            Ok((*v.get_unchecked(0), lhs))
        }
    }
}

impl<T: Real> Vector<T> {
    #[inline(always)]
    pub fn sum (&self, wait: impl Into<WaitList>) -> Result<Sum<T, &Self>> {
        Self::sum_by_deref(self, wait)
    }

    #[inline(always)]
    pub fn sum_by_deref<LHS: Deref<Target = Self>> (this: LHS, wait: impl Into<WaitList>) -> Result<Sum<T, LHS>> {
        Sum::new_custom(this, wait)
    }
}