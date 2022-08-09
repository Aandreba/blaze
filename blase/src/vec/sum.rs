use std::mem::MaybeUninit;
use std::{ops::Deref};
use blaze_rs::prelude::*;
use crate::{Real, max_work_group_size, utils::DerefCell};
use super::Vector;

pub struct Sum<T: Copy, LHS> {
    read: ReadBuffer<MaybeUninit<T>, DerefCell<Buffer<MaybeUninit<T>>>>,
    lhs: LHS
}

#[repr(transparent)]
pub struct SumWithSrc<T: Copy, LHS> (Sum<T, LHS>);

impl<T: Copy, LHS> Sum<T, LHS> {
    /// Returns an event that will resolve to the operations result, and also the will return the references to the oprands.
    /// Usefull when, for example, those references are [`Arc`s](std::sync::Arc) or [`MutexGuard`s](std::sync::MutexGuard)
    #[inline(always)]
    pub fn with_src (self) -> SumWithSrc<T, LHS> {
        SumWithSrc(self)
    }
}

impl<T: Real, LHS: Deref<Target = Vector<T>>> Event for Sum<T, LHS> {
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

impl<T: Real, LHS: Deref<Target = Vector<T>>> Event for SumWithSrc<T, LHS> {
    type Output = (T, LHS);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.0.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let v = self.0.read.consume(err)?;
        unsafe {
            Ok((v.get_unchecked(0).assume_init(), self.0.lhs))
        }
    }
}

lazy_static! {
    static ref WGS : usize = usize::max(max_work_group_size().get() / 2, 2);
}

impl<T: Real> Vector<T> {
    #[inline(always)]
    pub fn sum (&self, wait: impl Into<WaitList>) -> Result<Sum<T, &Self>> {
        Self::sum_by_deref(self, wait)
    }
 
    pub fn sum_by_deref<LHS: Deref<Target = Self>> (this: LHS, wait: impl Into<WaitList>) -> Result<Sum<T, LHS>> {
        let wgs = *WGS;
        let n = this.len()?;

        let temp_size = 2 * wgs;
        let mut temp_buffer = Buffer::<T>::new_uninit(temp_size, MemAccess::default(), false)?;
        let mut asum = Buffer::<T>::new_uninit(1, MemAccess::WRITE_ONLY, false)?;

        let (evt, (lhs, _)) : (RawEvent, (LHS, _)) = unsafe {
            let evt = T::vec_program().xasum(
                n as i32, 
                this,
                &mut temp_buffer,
                [wgs * temp_size], [wgs], 
                wait
            )?;

            (evt.to_raw(), evt.consume(None)?)
        };

        let (evt, _) : (RawEvent, _) = unsafe {
            let evt = T::vec_program().xasum_epilogue(&mut temp_buffer, &mut asum, [wgs], [wgs], WaitList::from_event(evt))?;
            (evt.to_raw(), evt.consume(None)?)
        };

        let read = Buffer::read_by_deref(
            DerefCell(asum), .., WaitList::from_event(evt)
        )?;

        Ok(Sum { read, lhs })
    }
}