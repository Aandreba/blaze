use std::{pin::Pin, ops::{RangeBounds, Deref}};
use crate::{core::*, event::{RawEvent, Event, WaitList}, context::Context, buffer::{Buffer, manager::{inner_write_from_ptr}}};

pub struct WriteBuffer<T: Copy, P: Deref<Target = [T]>> {
    event: RawEvent,
    #[allow(unused)]
    src: Pin<P>
}

impl<T: Copy + Unpin, P: Deref<Target = [T]>> WriteBuffer<T, P> {
    #[inline(always)]
    pub fn new<C: Context> (src: P, dst: &mut Buffer<T, C>, offset: usize, wait: impl Into<WaitList>) -> Result<Self> {
        let src = Pin::new(src);
        let range = offset..(offset + src.len());

        let event = unsafe { inner_write_from_ptr(dst, range, src.as_ptr(), wait).map(RawEvent::from_ptr)? };
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
pub unsafe fn write_from_ptr<T: Copy, C: Context> (src: *const T, dst: &mut Buffer<T, C>, range: impl RangeBounds<usize>, wait: impl Into<WaitList>) -> Result<RawEvent> {
    inner_write_from_ptr(dst, range, src, wait).map(RawEvent::from_ptr)
}

#[inline(always)]
pub fn write_from_static<T: Copy, C: Context> (src: &'static [T], dst: &mut Buffer<T, C>, offset: usize, wait: impl Into<WaitList>) -> Result<RawEvent> {
    let range = offset..(offset + src.len());
    unsafe { write_from_ptr(src.as_ptr(), dst, range, wait) }
}