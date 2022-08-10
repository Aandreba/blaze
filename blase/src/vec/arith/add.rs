use std::{mem::MaybeUninit, ops::Deref};
use blaze_rs::prelude::*;
use crate::{Real, utils::DerefCell, work_group_size, vec::EucVec};
use crate::vec::program::Add;

type OutputVec<T> = DerefCell<EucVec<MaybeUninit<T>>>;

pub struct Addition<T: Real, LHS, RHS> {
    evt: Add<LHS, RHS, OutputVec<T>, T>
}

pub struct AdditionWithSrc<T: Real, LHS, RHS> {
    evt: Add<LHS, RHS, OutputVec<T>, T>
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Addition<T, LHS, RHS> {
    #[inline]
    pub unsafe fn new_custom (lhs: LHS, rhs: RHS, len: usize, wait: impl Into<WaitList>) -> Result<Self> {
        let result = EucVec::new_uninit(len, false).map(DerefCell)?;
        let evt = T::vec_program().add(len, lhs, rhs, result, [work_group_size(len)], None, wait)?;
        Ok(Self { evt })
    }

    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> AdditionWithSrc<T, LHS, RHS> {
        AdditionWithSrc { evt: self.evt }
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for Addition<T, LHS, RHS> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.evt.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let (_, _, result) : (_, _, OutputVec<T>) = self.evt.consume(err)?;
        unsafe { Ok(result.0.assume_init()) }
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for AdditionWithSrc<T, LHS, RHS> {
    type Output = (EucVec<T>, LHS, RHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.evt.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let (lhs, rhs, result) : (_, _, OutputVec<T>) = self.evt.consume(err)?;
        unsafe { Ok((result.0.assume_init(), lhs, rhs)) }
    }
}

impl<T: Real> ::core::ops::Add<Self> for &EucVec<T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs, WaitList::EMPTY).unwrap().wait_unwrap()
    }
}

impl<T: Real> ::core::ops::Add<&EucVec<T>> for EucVec<T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn add(self, rhs: &EucVec<T>) -> Self::Output {
        &self + rhs
    }
}

impl<T: Real> ::core::ops::Add<EucVec<T>> for &EucVec<T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn add(self, rhs: EucVec<T>) -> Self::Output {
        self + &rhs
    }
}

impl<T: Real> ::core::ops::Add<Self> for EucVec<T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}