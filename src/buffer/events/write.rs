use std::{pin::Pin, ops::{RangeBounds, Deref}};
use crate::{core::*, event::{RawEvent, Event, WaitList}, buffer::{manager::{inner_write_from_ptr}, RawBuffer}};

pub struct WriteBuffer<T: Copy, P: Deref<Target = [T]>> {
    event: RawEvent,
    #[allow(unused)]
    src: Pin<P>
}

impl<T: Copy + Unpin, P: Deref<Target = [T]>> WriteBuffer<T, P> {
    #[inline(always)]
    pub unsafe fn new (src: P, dst: &mut RawBuffer, offset: usize, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let src = Pin::new(src);
        let range = offset..(offset + src.len());

        let event = inner_write_from_ptr(dst, range, src.as_ptr(), queue, wait).map(RawEvent::from_id)?;
        Ok(Self { event, src })
    }
}

impl<T: Copy + Unpin, P: Deref<Target = [T]>> Event for WriteBuffer<T, P> {
    type Output = ();

    #[inline(always)]
    fn consume (self) -> Self::Output {
       ()
    }
}

impl<T: Copy + Unpin, P: Deref<Target = [T]>> AsRef<RawEvent> for WriteBuffer<T, P> {
    #[inline(always)]
    fn as_ref(&self) -> &RawEvent {
        &self.event
    }
}

#[inline(always)]
pub unsafe fn write_from_ptr<T: Copy> (src: *const T, dst: &mut RawBuffer, range: impl RangeBounds<usize>, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
    inner_write_from_ptr(dst, range, src, queue, wait).map(RawEvent::from_id)
}

#[inline(always)]
pub unsafe fn write_from_static<T: Copy> (src: &'static [T], dst: &mut RawBuffer, offset: usize, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
    let range = offset..(offset + src.len());
    write_from_ptr(src.as_ptr(), dst, range, queue, wait)
}