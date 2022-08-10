use std::mem::MaybeUninit;
use std::ops::Mul;
use std::{ops::Deref};
use blaze_rs::event::various::Map;
use blaze_rs::prelude::*;
use crate::{Real, utils::DerefCell};
use super::{EucVec, sum::WGS};

pub type Magn<T, LHS> = Map<Dot<T, LHS, LHS>, fn(T) -> T>;
pub type Unit<T, LHS> = Map<DotWithSrc<T, LHS, LHS>, fn((T, LHS, LHS)) -> EucVec<T>>;

pub struct Dot<T: Copy, LHS, RHS> {
    read: ReadBuffer<MaybeUninit<T>, DerefCell<Buffer<MaybeUninit<T>>>>,
    lhs: LHS,
    rhs: RHS
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Dot<T, LHS, RHS> {
    pub unsafe fn new_custom (lhs: LHS, rhs: RHS, n: usize, wait: impl Into<WaitList>) -> Result<Self> {
        let wgs = WGS.get();
        let temp_size = 2 * wgs;
        let mut temp_buffer = Buffer::<T>::new_uninit(temp_size, MemAccess::default(), false)?;
        let mut asum = Buffer::<T>::new_uninit(1, MemAccess::WRITE_ONLY, false)?;

        let (evt, (lhs, rhs, _)) : (RawEvent, (LHS, RHS, _)) = {
            let evt = T::vec_program().xdot(
                n as i32, 
                lhs,
                rhs,
                &mut temp_buffer,
                [wgs * temp_size], [wgs],
                wait
            )?;

            (evt.to_raw(), evt.consume(None)?)
        };

        let (evt, _) : (RawEvent, _) = {
            let evt = T::vec_program().xasum_epilogue(&mut temp_buffer, &mut asum, [wgs], [wgs], WaitList::from_event(evt))?;
            (evt.to_raw(), evt.consume(None)?)
        };

        let read = Buffer::read_by_deref(
            DerefCell(asum), .., WaitList::from_event(evt)
        )?;

        Ok(Dot { read, lhs, rhs })
    }
}

#[repr(transparent)]
pub struct DotWithSrc<T: Copy, LHS, RHS> (Dot<T, LHS, RHS>);

impl<T: Copy, LHS, RHS> Dot<T, LHS, RHS> {
    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> DotWithSrc<T, LHS, RHS> {
        DotWithSrc(self)
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for Dot<T, LHS, RHS> {
    type Output = T;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.read.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let v = self.read.consume(err)?;
        unsafe {
            Ok(v.get_unchecked(0).assume_init())
        }
    }
} 

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for DotWithSrc<T, LHS, RHS> {
    type Output = (T, LHS, RHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.0.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let v = self.0.read.consume(err)?;
        unsafe {
            Ok((v.get_unchecked(0).assume_init(), self.0.lhs, self.0.rhs))
        }
    }
}

impl<T: Real> EucVec<T> {
    #[inline(always)]
    pub fn dot <RHS: Deref<Target = Self>> (&self, rhs: RHS, wait: impl Into<WaitList>) -> Result<Dot<T, &Self, RHS>> {
        Self::dot_by_deref(self, rhs, wait)
    }
    
    #[inline(always)]
    pub unsafe fn dot_unchecked <RHS: Deref<Target = Self>> (&self, rhs: RHS, wait: impl Into<WaitList>) -> Result<Dot<T, &Self, RHS>> {
        Self::dot_unchecked_by_deref(self, rhs, wait)
    }

    #[inline(always)]
    pub fn square_magn (&self, wait: impl Into<WaitList>) -> Result<Dot<T, &Self, &Self>> {
        Self::square_magn_by_deref(self, wait)
    }

    #[inline(always)]
    pub fn magn (&self, wait: impl Into<WaitList>) -> Result<Magn<T, &Self>> where T: num_traits::real::Real {
        Self::magn_by_deref(self, wait)
    }

    #[inline(always)]
    pub fn dot_by_deref<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (this: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<Dot<T, LHS, RHS>> {
        let len = this.len()?;
        let rhs_len = rhs.len()?;

        if len != rhs_len {
            return Err(Error::new(
                ErrorType::InvalidBufferSize,
                format!("Vectors must have the same length ({len} v. {rhs_len})")
            ));
        }

        unsafe {
            Dot::new_custom(this, rhs, len, wait)
        }
    }
    
    #[inline(always)]
    pub unsafe fn dot_unchecked_by_deref<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (this: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<Dot<T, LHS, RHS>> {
        let len = this.len()?;
        Dot::new_custom(this, rhs, len, wait)
    }

    #[inline(always)]
    pub fn square_magn_by_deref<LHS: Clone + Deref<Target = Self>> (this: LHS, wait: impl Into<WaitList>) -> Result<Dot<T, LHS, LHS>> {
        // SAFETY: This is safe because we are cloning a reference so the underlying vector is the same.
        unsafe {
            Self::dot_unchecked_by_deref(this.clone(), this, wait)
        }
    }

    #[inline(always)]
    pub fn magn_by_deref<LHS: Clone + Deref<Target = Self>> (this: LHS, wait: impl Into<WaitList>) -> Result<Magn<T, LHS>> where T: num_traits::real::Real {
        let square = Self::square_magn_by_deref(this, wait)?;
        square.map(T::sqrt)
    }
}

impl<T: Real> Mul<&EucVec<T>> for &EucVec<T> {
    type Output = T;

    #[inline(always)]
    fn mul(self, rhs: &EucVec<T>) -> Self::Output {
        self.dot(rhs, EMPTY).unwrap().wait_unwrap()
    }
}

impl<T: Real> Mul<EucVec<T>> for &EucVec<T> {
    type Output = T;

    #[inline(always)]
    fn mul(self, rhs: EucVec<T>) -> Self::Output {
        self * &rhs
    }
}

impl<T: Real> Mul<&EucVec<T>> for EucVec<T> {
    type Output = T;

    #[inline(always)]
    fn mul(self, rhs: &EucVec<T>) -> Self::Output {
        &self * rhs
    }
}

impl<T: Real> Mul<EucVec<T>> for EucVec<T> {
    type Output = T;

    #[inline(always)]
    fn mul(self, rhs: EucVec<T>) -> Self::Output {
        &self * &rhs
    }
}