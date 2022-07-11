use std::{pin::Pin, marker::PhantomData};
use crate::{core::*, event::{RawEvent, Event, WaitList}, buffer::{RawBuffer, IntoRange, BufferRange}};

pub struct ReadBuffer<'src, T: Copy> {
    event: RawEvent,
    dst: Pin<Vec<T>>,
    src: PhantomData<&'src RawBuffer>
}

impl<'src, T: Copy + Unpin> ReadBuffer<'src, T> {
    pub unsafe fn new (src: &'src RawBuffer, range: impl IntoRange, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let range = range.into_range::<T>(src)?;
        let mut result = Pin::new(Vec::with_capacity(range.cb / core::mem::size_of::<T>()));

        let event = src.read_to_ptr(range, result.as_mut_ptr(), queue, wait)?;
        Ok(Self { event, dst: result, src: PhantomData })
    }
}

impl<T: Copy + Unpin> Event for ReadBuffer<'_, T> {
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

#[repr(transparent)]
pub struct ReadBufferInto<'src, 'dst> {
    event: RawEvent,
    src: PhantomData<&'src [()]>,
    dst: PhantomData<&'dst mut RawBuffer>,
}

impl<'src, 'dst> ReadBufferInto<'src, 'dst> {
    pub unsafe fn new<T: Copy + Unpin> (src: &'src RawBuffer, offset: usize, dst: &'dst mut [T], queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let mut dst = Pin::new(dst);
        let range = BufferRange::from_parts::<T>(offset, dst.len()).unwrap();

        let event = src.read_to_ptr(range, dst.as_mut_ptr(), queue, wait)?;
        Ok(Self { event, src: PhantomData, dst: PhantomData })
    }
}

impl<'src, 'dst> Event for ReadBufferInto<'src, 'dst> {
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