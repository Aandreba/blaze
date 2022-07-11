use std::{pin::Pin, ops::{RangeBounds, Deref}};
use crate::{core::*, event::{RawEvent, Event, WaitList}, buffer::{RawBuffer}};

pub struct WriteBufferEvent<T: Copy, P: Deref<Target = [T]>> {
    event: RawEvent,
    #[allow(unused)]
    src: Pin<P>
}

impl<T: Copy + Unpin, P: Deref<Target = [T]>> WriteBufferEvent<T, P> {
    #[inline(always)]
    pub unsafe fn new (src: P, dst: &mut RawBuffer, offset: usize, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let src = Pin::new(src);
        let range = offset..(offset + src.len());

        let event = dst.write_from_ptr(range, src.as_ptr(), queue, wait)?;
        Ok(Self { event, src })
    }
}

impl<T: Copy + Unpin, P: Deref<Target = [T]>> Event for WriteBufferEvent<T, P> {
    type Output = ();

    #[inline(always)]
    fn as_raw(&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self) -> Self::Output {
       ()
    }
}

#[inline(always)]
pub unsafe fn write_from_ptr<T: Copy> (src: *const T, dst: &mut RawBuffer, range: impl RangeBounds<usize>, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
    dst.write_from_ptr(range, src, queue, wait)
}

#[inline(always)]
pub unsafe fn write_from_static<T: Copy> (src: &'static [T], dst: &mut RawBuffer, offset: usize, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
    let range = offset..(offset + src.len());
    write_from_ptr(src.as_ptr(), dst, range, queue, wait)
}