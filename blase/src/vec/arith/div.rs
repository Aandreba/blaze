use std::mem::MaybeUninit;
use std::ops::Deref;
use blaze_rs::prelude::*;
use crate::{Real, work_group_size};
use blaze_proc::docfg;
use crate::{utils::DerefCell, vec::Vector};
use crate::vec::{ScalDown, ScalDownInv};

type OutputVec<T> = DerefCell<Vector<MaybeUninit<T>>>;

pub struct Division<T: Real, LHS> {
    evt: ScalDown<LHS, OutputVec<T>, T>
}

pub struct DivisionWithSrc<T: Real, LHS> {
    evt: ScalDown<LHS, OutputVec<T>, T>
}

impl<T: Real, LHS: Deref<Target = Vector<T>>> Division<T, LHS> {
    #[inline]
    pub unsafe fn new_custom (lhs: LHS, alpha: T, len: usize, wait: impl Into<WaitList>) -> Result<Self> {
        let result = Vector::new_uninit(len, false).map(DerefCell)?;
        let evt = T::vec_program().scal_down(len, lhs, alpha, result, [work_group_size(len)], None, wait)?;
        Ok(Self { evt })
    }

    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> DivisionWithSrc<T, LHS> {
        DivisionWithSrc { evt: self.evt }
    }
}

impl<T: Real, RHS: Deref<Target = Vector<T>>> Event for Division<T, RHS> {
    type Output = Vector<T>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.evt.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let (_, result) : (_, OutputVec<T>) = self.evt.consume(err)?;
        unsafe { Ok(result.0.assume_init()) }
    }
}

impl<T: Real, RHS: Deref<Target = Vector<T>>> Event for DivisionWithSrc<T, RHS> {
    type Output = (Vector<T>, RHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.evt.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let (rhs, result) : (_, OutputVec<T>) = self.evt.consume(err)?;
        unsafe { Ok((result.0.assume_init(), rhs)) }
    }
}

impl<T: Real> ::core::ops::Div<T> for &Vector<T> {
    type Output = Vector<T>;

    #[inline(always)]
    fn div(self, rhs: T) -> Self::Output {
        self.div(rhs, WaitList::EMPTY).unwrap().wait_unwrap()
    }
}

impl<T: Real> ::core::ops::Div<&T> for &Vector<T> {
    type Output = Vector<T>;

    #[inline(always)]
    fn div(self, rhs: &T) -> Self::Output {
        self / *rhs
    }
}

impl<T: Real> ::core::ops::Div<T> for Vector<T> {
    type Output = Vector<T>;

    #[inline(always)]
    fn div(self, rhs: T) -> Self::Output {
        &self / rhs
    }
}

impl<T: Real> ::core::ops::Div<&T> for Vector<T> {
    type Output = Vector<T>;

    #[inline(always)]
    fn div(self, rhs: &T) -> Self::Output {
        &self / *rhs
    }
}

// INVERSE DIVISION
pub struct InvDivision<T: Real, RHS> {
    evt: ScalDownInv<RHS, OutputVec<T>, T>
}

pub struct InvDivisionWithSrc<T: Real, RHS> {
    evt: ScalDownInv<RHS, OutputVec<T>, T>
}

impl<T: Real, RHS: Deref<Target = Vector<T>>> InvDivision<T, RHS> {
    #[inline]
    pub unsafe fn new_custom (alpha: T, rhs: RHS, len: usize, wait: impl Into<WaitList>) -> Result<Self> {
        let result = Vector::new_uninit(len, false).map(DerefCell)?;
        let evt = T::vec_program().scal_down_inv(len, alpha, rhs, result, [work_group_size(len)], None, wait)?;
        Ok(Self { evt })
    }

    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> InvDivisionWithSrc<T, RHS> {
        InvDivisionWithSrc { evt: self.evt }
    }
}

impl<T: Real, RHS: Deref<Target = Vector<T>>> Event for InvDivision<T, RHS> {
    type Output = Vector<T>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.evt.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let (_, result) : (_, OutputVec<T>) = self.evt.consume(err)?;
        unsafe { Ok(result.0.assume_init()) }
    }
}

impl<T: Real, RHS: Deref<Target = Vector<T>>> Event for InvDivisionWithSrc<T, RHS> {
    type Output = (Vector<T>, RHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.evt.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let (rhs, result) : (_, OutputVec<T>) = self.evt.consume(err)?;
        unsafe { Ok((result.0.assume_init(), rhs)) }
    }
}

macro_rules! impl_div {
    ($($(#[cfg(feature = $feat:literal)])? $t:ty),+) => {
        $(
            $(#[docfg(feature = $feat)])?
            impl ::core::ops::Div<&Vector<$t>> for $t {
                type Output = Vector<$t>;
            
                #[inline(always)]
                fn div(self, rhs: &Vector<$t>) -> Self::Output {
                    rhs.div_inv(self, WaitList::EMPTY).unwrap().wait_unwrap()
                }
            }
            
            $(#[docfg(feature = $feat)])?
            impl ::core::ops::Div<Vector<$t>> for $t {
                type Output = Vector<$t>;
            
                #[inline(always)]
                fn div(self, rhs: Vector<$t>) -> Self::Output {
                    self / &rhs
                }
            }

            $(#[docfg(feature = $feat)])?
            impl ::core::ops::Div<&Vector<$t>> for &$t {
                type Output = Vector<$t>;
            
                #[inline(always)]
                fn div(self, rhs: &Vector<$t>) -> Self::Output {
                    *self / rhs
                }
            }
            
            $(#[docfg(feature = $feat)])?
            impl ::core::ops::Div<Vector<$t>> for &$t {
                type Output = Vector<$t>;
            
                #[inline(always)]
                fn div(self, rhs: Vector<$t>) -> Self::Output {
                    *self / &rhs
                }
            }
        )+
    };
}

impl_div! {
    u8, u16, u32, u64,
    i8, i16, i32, i64,
    #[cfg(feature = "half")] ::half::f16,
    f32, 
    #[cfg(feature = "double")] f64
}