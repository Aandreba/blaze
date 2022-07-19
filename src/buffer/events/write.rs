use std::{ops::{Deref, DerefMut}, pin::Pin};
use crate::{core::*, event::{RawEvent, Event, WaitList}, buffer::{BufferRange, Buffer}, prelude::Context};

pub struct WriteBuffer<Src, Dst> {
    event: RawEvent,
    src: Pin<Src>,
    dst: Dst
}

impl<T: Copy + Unpin, Src: Deref<Target = [T]>, Dst: DerefMut<Target = Buffer<T, C>>, C: Context> WriteBuffer<Src, Dst> {
    #[inline(always)]
    pub unsafe fn new (src: Src, offset: usize, mut dst: Dst, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let src = Pin::new(src);
        let range = BufferRange::from_parts::<T>(offset, dst.size()?).unwrap();
        let event = dst.write_from_ptr(range, src.as_ptr(), queue, wait)?;
        Ok(Self { event, src, dst })
    }
}

impl<T: Copy + Unpin, Src: Deref<Target = [T]>, Dst: DerefMut<Target = Buffer<T, C>>, C: Context> Event for WriteBuffer<Src, Dst> {
    type Output = (Src, Dst);

    #[inline(always)]
    fn as_raw(&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, error: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = error { return Err(err); }
        Ok((Pin::into_inner(self.src), self.dst))
    }
}