use std::{mem::{MaybeUninit, transmute}, ops::Deref, cmp::Ordering};
use blaze_rs::prelude::*;
use crate::{utils::DerefCell, Real, work_group_size};
use super::EucVec;

pub struct LaneOrd<LHS, RHS> {
    evt: ReadBuffer<MaybeUninit<i8>, DerefCell<Buffer<MaybeUninit<i8>>>>,
    lhs: LHS,
    rhs: RHS
}

#[repr(transparent)]
pub struct LaneOrdWithSrc<LHS, RHS> (LaneOrd<LHS, RHS>);

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> LaneOrd<LHS, RHS> {
    pub unsafe fn new_custom (lhs: LHS, rhs: RHS, len: usize, wait: impl Into<WaitList>) -> Result<Self> {
        let mut result = Buffer::<i8>::new_uninit(len, MemAccess::WRITE_ONLY, false)?;
        
        let (evt, (lhs, rhs, _)) : (RawEvent, (LHS, RHS, _)) = {
            let evt = T::vec_program().vec_cmp_ord(len, lhs, rhs, &mut result, [work_group_size(len)], None, wait)?;
            (evt.to_raw(), evt.consume(None)?)
        };

        let evt = Buffer::read_by_deref(DerefCell(result), .., WaitList::from_event(evt))?;
        Ok(Self { evt, lhs, rhs })
    }

    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> LaneOrdWithSrc<LHS, RHS> {
        LaneOrdWithSrc(self)
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for LaneOrd<LHS, RHS> {
    type Output = Vec<Ordering>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.evt.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let out : Vec<MaybeUninit<i8>> = self.evt.consume(err)?;
        unsafe {
            Ok(transmute(out))
        }
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for LaneOrdWithSrc<LHS, RHS> {
    type Output = (Vec<Ordering>, LHS, RHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.0.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let out : Vec<MaybeUninit<i8>> = self.0.evt.consume(err)?;

        unsafe {
            Ok((transmute(out), self.0.lhs, self.0.rhs))
        }
    }
}

impl<T: Real> EucVec<T> {
    /// Performs a total comparison of the elements in the vector's lanes.\
    /// For floats, [`total_cmp`](::std::primitive::f32::total_cmp) is used.
    #[inline(always)]
    pub fn lane_ord <RHS: Deref<Target = Self>> (&self, rhs: RHS, wait: impl Into<WaitList>) -> Result<LaneOrd<&Self, RHS>> {
        Self::lane_ord_by_deref(self, rhs, wait)
    }

    /// Performs a total comparison of the elements in the vector's lanes, without checking if they have the same len.\
    /// For floats, [`total_cmp`](::std::primitive::f32::total_cmp) is used.
    /// # Safety
    /// This function is only safe if the vectors have the same len.
    #[inline(always)]
    pub unsafe fn lane_ord_unchecked<RHS: Deref<Target = Self>> (&self, rhs: RHS, wait: impl Into<WaitList>) -> Result<LaneOrd<&Self, RHS>> {
        Self::lane_ord_unchecked_by_deref(self, rhs, wait)
    }

    #[inline(always)]
    pub fn lane_ord_by_deref <LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (lhs: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<LaneOrd<LHS, RHS>> {
        let lhs_len = lhs.len()?;
        let rhs_len = rhs.len()?;

        if lhs_len != rhs_len {
            return Err(Error::new(ErrorType::InvalidBufferSize, format!("Vectors must be of the same length ({lhs_len} v. {rhs_len})")));
        }

        unsafe {
            LaneOrd::new_custom(lhs, rhs, lhs_len, wait)
        }
    }

    #[inline(always)]
    pub unsafe fn lane_ord_unchecked_by_deref <LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (lhs: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<LaneOrd<LHS, RHS>> {
        let len = lhs.len()?;
        LaneOrd::new_custom(lhs, rhs, len, wait)
    }
}

/* PARTIAL ORDERING */
pub struct LanePartialOrd<LHS, RHS> {
    evt: ReadBuffer<MaybeUninit<i8>, DerefCell<Buffer<MaybeUninit<i8>>>>,
    lhs: LHS,
    rhs: RHS
}

#[repr(transparent)]
pub struct LanePartialOrdWithSrc<LHS, RHS> (LanePartialOrd<LHS, RHS>);

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> LanePartialOrd<LHS, RHS> {
    pub unsafe fn new_custom (lhs: LHS, rhs: RHS, len: usize, wait: impl Into<WaitList>) -> Result<Self> {
        let mut result = Buffer::<i8>::new_uninit(len, MemAccess::WRITE_ONLY, false)?;
        
        let (evt, (lhs, rhs, _)) : (RawEvent, (LHS, RHS, _)) = {
            let evt = T::vec_program().vec_cmp_partial_ord(len, lhs, rhs, &mut result, [work_group_size(len)], None, wait)?;
            (evt.to_raw(), evt.consume(None)?)
        };

        let evt = Buffer::read_by_deref(DerefCell(result), .., WaitList::from_event(evt))?;
        Ok(Self { evt, lhs, rhs })
    }

    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> LanePartialOrdWithSrc<LHS, RHS> {
        LanePartialOrdWithSrc(self)
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for LanePartialOrd<LHS, RHS> {
    type Output = Vec<Option<Ordering>>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.evt.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let out : Vec<MaybeUninit<i8>> = self.evt.consume(err)?;
        unsafe {
            Ok(transmute(out))
        }
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for LanePartialOrdWithSrc<LHS, RHS> {
    type Output = (Vec<Option<Ordering>>, LHS, RHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.0.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let out : Vec<MaybeUninit<i8>> = self.0.evt.consume(err)?;

        unsafe {
            Ok((transmute(out), self.0.lhs, self.0.rhs))
        }
    }
}

impl<T: Real> EucVec<T> {
    /// Performs a partial comparison of the elements in the vector's lanes.
    #[inline(always)]
    pub fn lane_partial_ord<RHS: Deref<Target = Self>> (&self, rhs: RHS, wait: impl Into<WaitList>) -> Result<LanePartialOrd<&Self, RHS>> {
        Self::lane_partial_ord_by_deref(self, rhs, wait)
    }

    /// Performs a partial comparison of the elements in the vector's lanes, without checking if they have the same len.
    /// # Safety
    /// This function is only safe if the vectors have the same len.
    #[inline(always)]
    pub unsafe fn lane_partial_ord_unchecked<RHS: Deref<Target = Self>> (&self, rhs: RHS, wait: impl Into<WaitList>) -> Result<LanePartialOrd<&Self, RHS>> {
        Self::lane_partial_ord_unchecked_by_deref(self, rhs, wait)
    }

    #[inline(always)]
    pub fn lane_partial_ord_by_deref <LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (lhs: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<LanePartialOrd<LHS, RHS>> {
        let lhs_len = lhs.len()?;
        let rhs_len = rhs.len()?;

        if lhs_len != rhs_len {
            return Err(Error::new(ErrorType::InvalidBufferSize, format!("Vectors must be of the same length ({lhs_len} v. {rhs_len})")));
        }

        unsafe {
            LanePartialOrd::new_custom(lhs, rhs, lhs_len, wait)
        }
    }

    #[inline(always)]
    pub unsafe fn lane_partial_ord_unchecked_by_deref <LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (lhs: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<LanePartialOrd<LHS, RHS>> {
        let len = lhs.len()?;
        LanePartialOrd::new_custom(lhs, rhs, len, wait)
    }
}