use std::{pin::Pin, ops::{RangeBounds, DerefMut}};
use crate::{core::*, event::{RawEvent, Event, WaitList}, buffer::{RawBuffer, range_len}};

pub struct ReadBufferEvent<T: Copy> {
    event: RawEvent,
    result: Pin<Vec<T>>
}

impl<T: Copy + Unpin> ReadBufferEvent<T> {
    pub unsafe fn new (src: &RawBuffer, range: impl RangeBounds<usize>, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let len = range_len(core::mem::size_of::<T>(), &range);
        let mut result = Pin::new(Vec::with_capacity(len));

        let event = src.read_to_ptr(range, result.as_mut_ptr(), queue, wait)?;
        Ok(Self { event, result })
    }
}

impl<T: Copy + Unpin> Event for ReadBufferEvent<T> {
    type Output = Vec<T>;

    #[inline(always)]
    fn as_raw(&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        let mut result = Pin::into_inner(self.result);
        unsafe { result.set_len(result.capacity()) }
        Ok(result)
    }
}

pub struct ReadBufferInto<T: Copy, P: DerefMut<Target = [T]>> {
    event: RawEvent,
    #[allow(unused)]
    dst: Pin<P>
}

impl<T: Copy + Unpin, P: DerefMut<Target = [T]>> ReadBufferInto<T, P> {
    pub unsafe fn new (src: &RawBuffer, dst: P, offset: usize, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let mut dst = Pin::new(dst);
        let range = offset..(offset + dst.len());

        let event = src.read_to_ptr(range, dst.as_mut_ptr(), queue, wait)?;
        Ok(Self { event, dst })
    }
}

impl<T: Copy + Unpin, P: DerefMut<Target = [T]>> Event for ReadBufferInto<T, P> {
    type Output = ();

    #[inline(always)]
    fn as_raw(&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, error: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = error { return Err(err); }
        Ok(())
    }
}

impl<T: Copy, P: DerefMut<Target = [T]>> AsRef<RawEvent> for ReadBufferInto<T, P> {
    #[inline(always)]
    fn as_ref(&self) -> &RawEvent {
        &self.event
    }
}