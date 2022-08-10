use std::mem::MaybeUninit;
use std::ops::Deref;
use blaze_rs::prelude::*;
use blaze_proc::docfg;
use crate::{Real, work_group_size};
use crate::{utils::DerefCell, vec::EucVec};
use crate::vec::Scal;

type OutputVec<T> = DerefCell<EucVec<MaybeUninit<T>>>;

pub struct Scale<T: Real, RHS> {
    evt: Scal<RHS, OutputVec<T>, T>
}

pub struct ScaleWithSrc<T: Real, RHS> {
    evt: Scal<RHS, OutputVec<T>, T>
}

impl<T: Real, RHS: Deref<Target = EucVec<T>>> Scale<T, RHS> {
    #[inline]
    pub unsafe fn new_custom (alpha: T, rhs: RHS, len: usize, wait: impl Into<WaitList>) -> Result<Self> {
        let result = EucVec::new_uninit(len, false).map(DerefCell)?;
        let evt = T::vec_program().scal(len, alpha, rhs, result, [work_group_size(len)], None, wait)?;
        Ok(Self { evt })
    }

    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> ScaleWithSrc<T, RHS> {
        ScaleWithSrc { evt: self.evt }
    }
}

impl<T: Real, RHS: Deref<Target = EucVec<T>>> Event for Scale<T, RHS> {
    type Output = EucVec<T>;

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

impl<T: Real, RHS: Deref<Target = EucVec<T>>> Event for ScaleWithSrc<T, RHS> {
    type Output = (EucVec<T>, RHS);

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

impl<T: Real> ::core::ops::Mul<T> for &EucVec<T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn mul(self, rhs: T) -> Self::Output {
        self.mul(rhs, WaitList::EMPTY).unwrap().wait_unwrap()
    }
}

impl<T: Real> ::core::ops::Mul<&T> for &EucVec<T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn mul(self, rhs: &T) -> Self::Output {
        self * *rhs
    }
}

impl<T: Real> ::core::ops::Mul<T> for EucVec<T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn mul(self, rhs: T) -> Self::Output {
        &self * rhs
    }
}

impl<T: Real> ::core::ops::Mul<&T> for EucVec<T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn mul(self, rhs: &T) -> Self::Output {
        &self * *rhs
    }
}

macro_rules! impl_mul {
    ($($(#[cfg(feature = $feat:literal)])? $t:ty),+) => {
        $(
            $(#[docfg(feature = $feat)])?
            impl ::core::ops::Mul<&EucVec<$t>> for $t {
                type Output = EucVec<$t>;
            
                #[inline(always)]
                fn mul(self, rhs: &EucVec<$t>) -> Self::Output {
                    rhs * self
                }
            }
            
            $(#[docfg(feature = $feat)])?
            impl ::core::ops::Mul<EucVec<$t>> for $t {
                type Output = EucVec<$t>;
            
                #[inline(always)]
                fn mul(self, rhs: EucVec<$t>) -> Self::Output {
                    self * &rhs
                }
            }

            $(#[docfg(feature = $feat)])?
            impl ::core::ops::Mul<&EucVec<$t>> for &$t {
                type Output = EucVec<$t>;
            
                #[inline(always)]
                fn mul(self, rhs: &EucVec<$t>) -> Self::Output {
                    *self * rhs
                }
            }
            
            $(#[docfg(feature = $feat)])?
            impl ::core::ops::Mul<EucVec<$t>> for &$t {
                type Output = EucVec<$t>;
            
                #[inline(always)]
                fn mul(self, rhs: EucVec<$t>) -> Self::Output {
                    *self * &rhs
                }
            }
        )+
    };
}

impl_mul! {
    u8, u16, u32, u64,
    i8, i16, i32, i64,
    #[cfg(feature = "half")] ::half::f16,
    f32, 
    #[cfg(feature = "double")] f64
}