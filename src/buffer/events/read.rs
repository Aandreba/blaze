use std::{pin::Pin, ops::{RangeBounds, Deref, DerefMut}};
use crate::{core::*, event::{RawEvent, Event}, context::Context, buffer::{Buffer, manager::{range_len, read_to_ptr}}};

pub struct ReadBuffer<T: Copy> {
    event: RawEvent,
    result: Pin<Vec<T>>
}

impl<T: Copy + Unpin> ReadBuffer<T> {
    pub fn new<C: Context> (src: &Buffer<T, C>, range: impl RangeBounds<usize>) -> Result<Self> {
        let len = range_len(src, &range)?;
        let mut result = Pin::new(Vec::with_capacity(len));

        unsafe {
            let event = read_to_ptr(src, range, result.as_mut_ptr()).map(RawEvent::from_ptr)?;
            Ok(Self { event, result })
        }
    }
}

impl<T: Copy + Unpin> Event for ReadBuffer<T> {
    type Output = Vec<T>;

    #[inline(always)]
    fn consume (self) -> Self::Output {
        Pin::into_inner(self.result)
    }
}

impl<T: Copy> AsRef<RawEvent> for ReadBuffer<T> {
    #[inline(always)]
    fn as_ref(&self) -> &RawEvent {
        &self.event
    }
}

impl<T: Copy> Deref for ReadBuffer<T> {
    type Target = RawEvent;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<T: Copy> DerefMut for ReadBuffer<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}