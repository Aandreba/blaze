use std::{pin::Pin, ops::{Deref, DerefMut}};
use crate::{core::*, event::{RawEvent, Event, WaitList}, buffer::{IntoRange, BufferRange, Buffer}, prelude::Context};

pub struct ReadBuffer<T: Copy, Src> {
    event: RawEvent,
    dst: Pin<Vec<T>>,
    src: Src
}

impl<T: Copy + Unpin, Src: Deref<Target = Buffer<T, C>>, C: Context> ReadBuffer<T, Src> {
    pub unsafe fn new (src: Src, range: impl IntoRange, queue: &RawCommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let range = range.into_range::<T>(&src)?;
        let mut result = Pin::new(Vec::with_capacity(range.cb / core::mem::size_of::<T>()));

        let event = src.read_to_ptr(range, result.as_mut_ptr(), queue, wait)?;
        Ok(Self { event, dst: result, src })
    }
}

impl<T: Copy + Unpin, Src: Deref<Target = Buffer<T, C>>, C: Context> Event for ReadBuffer<T, Src> {
    type Output = Vec<T>;

    #[inline(always)]
    fn as_raw(&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        let mut result = Pin::into_inner(self.dst);
        unsafe { result.set_len(result.capacity()) }
        Ok(result)
    }
}

pub struct ReadBufferInto<Src, Dst> {
    event: RawEvent,
    src: Src,
    dst: Pin<Dst>
}

impl<T: Copy + Unpin, Src: Deref<Target = Buffer<T, C>>, Dst: DerefMut<Target = [T]>, C: Context> ReadBufferInto<Src, Dst> {
    pub unsafe fn new (src: Src, offset: usize, dst: Dst, queue: &RawCommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let mut dst = Pin::new(dst);
        let range = BufferRange::from_parts::<T>(offset, dst.len()).unwrap();

        let event = src.read_to_ptr(range, dst.as_mut_ptr(), queue, wait)?;
        Ok(Self { event, src, dst })
    }
}

impl<T: Copy + Unpin, Src: Deref<Target = Buffer<T, C>>, Dst: DerefMut<Target = [T]>, C: Context> Event for ReadBufferInto<Src, Dst> {
    type Output = (Src, Dst);

    #[inline(always)]
    fn as_raw(&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, error: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = error { return Err(err); }
        Ok((self.src, Pin::into_inner(self.dst)))
    }
}