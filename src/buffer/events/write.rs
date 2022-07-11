use std::{marker::PhantomData};
use crate::{core::*, event::{RawEvent, Event, WaitList}, buffer::{RawBuffer, IntoRange, BufferRange}};

#[repr(transparent)]
pub struct WriteBuffer<'src, 'dst> {
    event: RawEvent,
    src: PhantomData<&'src [()]>,
    dst: PhantomData<&'dst mut RawBuffer>
}

impl<'src, 'dst> WriteBuffer<'src, 'dst> {
    #[inline(always)]
    pub unsafe fn new<T: Copy + Unpin> (src: &'src [T], offset: usize, dst: &'dst mut RawBuffer, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let range = BufferRange::from_parts::<T>(offset, dst.size()?).unwrap();
        let event = dst.write_from_ptr(range, src.as_ptr(), queue, wait)?;
        Ok(Self { event, src: PhantomData, dst: PhantomData })
    }
}

impl<'src, 'dst> Event for WriteBuffer<'src, 'dst> {
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