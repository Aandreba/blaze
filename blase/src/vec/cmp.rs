use std::{ops::Deref, mem::MaybeUninit};
use bitvec::prelude::BitBox;
use blaze_rs::prelude::*;
use elor::Either;
use crate::{Real, utils::DerefCell, work_group_size};
use super::{EucVec};

pub struct LaneEq<LHS, RHS> {
    evt: ReadBuffer<MaybeUninit<u32>, DerefCell<Buffer<MaybeUninit<u32>>>>,
    lhs: LHS,
    rhs: RHS
}

#[repr(transparent)]
pub struct LaneEqWithSrc<LHS, RHS> (LaneEq<LHS, RHS>);

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> LaneEq<LHS, RHS> {
    pub unsafe fn new_custom (lhs: LHS, rhs: RHS, len: usize, wait: impl Into<WaitList>) -> Result<Self> {
        let result_len = len.div_ceil(u32::BITS as usize);
        let mut result = Buffer::<u32>::new_uninit(result_len, MemAccess::WRITE_ONLY, false)?;
        
        let (evt, (lhs, rhs, _)) : (RawEvent, (LHS, RHS, _)) = {
            let evt = T::vec_program().vec_cmp_eq(len, lhs, rhs, &mut result, [work_group_size(len)], None, wait)?;
            (evt.to_raw(), evt.consume(None)?)
        };

        let evt = Buffer::read_by_deref(DerefCell(result), .., WaitList::from_event(evt))?;
        Ok(Self { evt, lhs, rhs })
    }

    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> LaneEqWithSrc<LHS, RHS> {
        LaneEqWithSrc(self)
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for LaneEq<LHS, RHS> {
    type Output = (BitBox<u32>, usize);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.evt.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let len = self.lhs.len()?;
        let out : Vec<MaybeUninit<u32>> = self.evt.consume(err)?;

        let slice = unsafe {
            out.into_boxed_slice().assume_init()
        };

        Ok((BitBox::from_boxed_slice(slice), len))
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for LaneEqWithSrc<LHS, RHS> {
    type Output = (BitBox<u32>, LHS, RHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.0.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let out : Vec<MaybeUninit<u32>> = self.0.evt.consume(err)?;
        let slice = unsafe {
            out.into_boxed_slice().assume_init()
        };

        Ok((BitBox::from_boxed_slice(slice), self.0.lhs, self.0.rhs))
    }
}

impl<T: Real> EucVec<T> {
    #[inline(always)]
    pub fn lane_eq <RHS: Deref<Target = Self>> (&self, rhs: RHS, wait: impl Into<WaitList>) -> Result<LaneEq<&Self, RHS>> {
        Self::lane_eq_by_deref(self, rhs, wait)
    }

    #[inline(always)]
    pub unsafe fn lane_eq_unchecked<RHS: Deref<Target = Self>> (&self, rhs: RHS, wait: impl Into<WaitList>) -> Result<LaneEq<&Self, RHS>> {
        Self::lane_eq_unchecked_by_deref(self, rhs, wait)
    }

    #[inline(always)]
    pub fn lane_eq_by_deref <LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (lhs: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<LaneEq<LHS, RHS>> {
        let lhs_len = lhs.len()?;
        let rhs_len = rhs.len()?;

        if lhs_len != rhs_len {
            return Err(Error::new(ErrorType::InvalidBufferSize, format!("Vectors must be of the same length ({lhs_len} v. {rhs_len})")));
        }

        unsafe {
            LaneEq::new_custom(lhs, rhs, lhs_len, wait)
        }
    }

    #[inline(always)]
    pub unsafe fn lane_eq_unchecked_by_deref <LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (lhs: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<LaneEq<LHS, RHS>> {
        let len = lhs.len()?;
        LaneEq::new_custom(lhs, rhs, len, wait)
    }
}

pub struct VecEq<LHS, RHS> (Either<LaneEq<LHS, RHS>, (FlagEvent, LHS, RHS)>);
pub struct VecEqWithSrc<LHS, RHS> (Either<LaneEqWithSrc<LHS, RHS>, (FlagEvent, LHS, RHS)>);

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> VecEq<LHS, RHS> {
    pub fn new (lhs: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<Self> {
        if lhs.eq_buffer(&rhs) {
            let flag = FlagEvent::new()?;
            flag.complete(None)?;
            return Ok(VecEq(Either::Right((flag, lhs, rhs))))
        }

        let lhs_len = lhs.len()?;
        let rhs_len = rhs.len()?;
        if lhs_len != rhs_len {
            return Err(Error::new(ErrorType::InvalidBufferSize, format!("Vectors must be of the same length ({lhs_len} v. {rhs_len})")));
        }

        let lane = unsafe {
            LaneEq::new_custom(lhs, rhs, lhs_len, wait)?
        };
        
        Ok(VecEq(Either::Left(lane)))
    }

    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> VecEqWithSrc<LHS, RHS> {
        match self.0 {
            Either::Left(lane) => VecEqWithSrc(Either::Left(lane.with_src())),
            Either::Right(x) => VecEqWithSrc(Either::Right(x)),
        }
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for VecEq<LHS, RHS> {
    type Output = bool;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        match &self.0 {
            Either::Left(evt) => evt.as_raw(),
            Either::Right((evt, _, _)) => evt.as_raw()
        }
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        match self.0 {
            Either::Left(lane) => {
                let (inner, len) = lane.consume(err)?;
                Ok(inner.into_iter().take(len).all(|x| x))
            },

            Either::Right((flag, _, _)) => {
                flag.consume(err)?;
                Ok(true)
            }
        }
    }
}

impl<T: Real, LHS: Deref<Target = EucVec<T>>, RHS: Deref<Target = EucVec<T>>> Event for VecEqWithSrc<LHS, RHS> {
    type Output = (bool, LHS, RHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        match &self.0 {
            Either::Left(evt) => evt.as_raw(),
            Either::Right((evt, _, _)) => evt.as_raw()
        }
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        match self.0 {
            Either::Left(lane) => {
                let (inner, lhs, rhs) = lane.consume(err)?;
                let len = lhs.len()?;
                Ok((inner.into_iter().take(len).all(|x| x), lhs, rhs))
            },

            Either::Right((flag, lhs, rhs)) => {
                flag.consume(err)?;
                Ok((true, lhs, rhs))
            }
        }
    }
}

impl<T: Real> EucVec<T> {
    #[inline(always)]
    pub fn eq<RHS: Deref<Target = Self>> (&self, rhs: RHS, wait: impl Into<WaitList>) -> Result<VecEq<&Self, RHS>> {
        Self::eq_by_deref(self, rhs, wait)
    }

    #[inline(always)]
    pub fn eq_by_deref<LHS: Deref<Target = Self>, RHS: Deref<Target = Self>> (lhs: LHS, rhs: RHS, wait: impl Into<WaitList>) -> Result<VecEq<LHS, RHS>> {
        VecEq::new(lhs, rhs, wait)
    }
}

impl<T: Real> PartialEq for EucVec<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.eq(other, EMPTY).unwrap().wait_unwrap()
    }
}

impl<T: Real + Eq> Eq for EucVec<T> {}