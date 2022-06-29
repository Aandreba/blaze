use std::{pin::Pin, ops::{RangeBounds, DerefMut}};
use crate::{core::*, event::{RawEvent, Event, WaitList}, context::Context, buffer::{manager::{range_len, inner_read_to_ptr}, RawBuffer}};

pub struct ReadBuffer<T: Copy> {
    event: RawEvent,
    result: Pin<Vec<T>>
}

impl<T: Copy + Unpin> ReadBuffer<T> {
    pub fn new<C: Context> (src: &RawBuffer, range: impl RangeBounds<usize>, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let len = range_len(src, core::mem::size_of::<T>(), &range);
        let mut result = Pin::new(Vec::with_capacity(len));

        unsafe {
            let event = inner_read_to_ptr(src, range, result.as_mut_ptr(), queue, wait).map(RawEvent::from_id)?;
            Ok(Self { event, result })
        }
    }
}

impl<T: Copy + Unpin> Event for ReadBuffer<T> {
    type Output = Vec<T>;

    #[inline(always)]
    fn consume (self) -> Self::Output {
        let mut result = Pin::into_inner(self.result);
        unsafe { result.set_len(result.capacity()) }
        result
    }
}

impl<T: Copy> AsRef<RawEvent> for ReadBuffer<T> {
    #[inline(always)]
    fn as_ref(&self) -> &RawEvent {
        &self.event
    }
}

pub struct ReadBufferInto<T: Copy, P: DerefMut<Target = [T]>> {
    event: RawEvent,
    #[allow(unused)]
    dst: Pin<P>
}

impl<T: Copy + Unpin, P: DerefMut<Target = [T]>> ReadBufferInto<T, P> {
    pub fn new<C: Context> (src: &RawBuffer, dst: P, offset: usize, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let mut dst = Pin::new(dst);
        let range = offset..(offset + dst.len());

        unsafe {
            let event = inner_read_to_ptr(src, range, dst.as_mut_ptr(), queue, wait).map(RawEvent::from_id)?;
            Ok(Self { event, dst })
        }
    }
}

impl<T: Copy + Unpin, P: DerefMut<Target = [T]>> Event for ReadBufferInto<T, P> {
    type Output = ();

    #[inline(always)]
    fn consume (self) -> Self::Output {
       ()
    }
}

impl<T: Copy, P: DerefMut<Target = [T]>> AsRef<RawEvent> for ReadBufferInto<T, P> {
    #[inline(always)]
    fn as_ref(&self) -> &RawEvent {
        &self.event
    }
}