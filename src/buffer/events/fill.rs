use std::marker::PhantomData;
use crate::{buffer::{RawBuffer, IntoRange}, prelude::*, event::WaitList};

#[cfg_attr(docsrs, doc(cfg(feature = "cl1_2")))]
#[repr(transparent)]
pub struct FillBuffer<'dst> {
    event: RawEvent,
    dst: PhantomData<&'dst mut RawBuffer>
}

impl<'dst> FillBuffer<'dst> {
    #[inline(always)]
    pub unsafe fn new<T: Copy> (src: T, dst: &'dst mut RawBuffer, range: impl IntoRange, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let event = dst.fill(src, range, queue, wait)?;
        Ok(Self { event, dst: PhantomData })
    }
}

impl<'dst> Event for FillBuffer<'dst> {
    type Output = ();

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) }
        Ok(())
    }
}