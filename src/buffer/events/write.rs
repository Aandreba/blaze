use std::{pin::Pin, ops::{RangeBounds, Deref}};
use crate::{core::*, event::{RawEvent, Event}, context::Context, buffer::{Buffer, manager::{write_from_ptr}}};

pub struct WriteBuffer<T: Copy, P: Deref<Target = [T]>> {
    event: RawEvent,
    #[allow(unused)]
    src: Pin<P>
}

impl<T: Copy + Unpin, P: Deref<Target = [T]>> WriteBuffer<T, P> {
    #[inline(always)]
    pub fn new<C: Context> (src: P, dst: &mut Buffer<T, C>, offset: usize) -> Result<Self> {
        let src = Pin::new(src);
        let range = offset..(offset + src.len());

        let event = unsafe { write_from_ptr(dst, range, src.as_ptr()).map(RawEvent::from_ptr)? };
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

#[derive(Clone)]
pub struct WriteBufferStatic {
    event: RawEvent
}

impl WriteBufferStatic {
    #[inline(always)]
    pub unsafe fn new_from_ptr<T: Copy, C: Context> (src: *const T, dst: &mut Buffer<T, C>, range: impl RangeBounds<usize>) -> Result<Self> {
        let event = write_from_ptr(dst, range, src).map(RawEvent::from_ptr)?;
        Ok(Self { event })
    }

    #[inline(always)]
    pub fn new_from_static<T: Copy, C: Context> (src: &'static [T], dst: &mut Buffer<T, C>, offset: usize) -> Result<Self> {
        let range = offset..(offset + src.len());
        unsafe { Self::new_from_ptr(src.as_ptr(), dst, range) }
    }
}

impl Event for WriteBufferStatic {
    type Output = ();

    #[inline(always)]
    fn consume (self) -> Self::Output {
       ()
    }
}

impl AsRef<RawEvent> for WriteBufferStatic {
    #[inline(always)]
    fn as_ref(&self) -> &RawEvent {
        &self.event
    }
}