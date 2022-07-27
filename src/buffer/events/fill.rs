use std::{ops::DerefMut};
use crate::{buffer::{RawBuffer, IntoRange, Buffer}, prelude::*, event::WaitList};

#[cfg_attr(docsrs, doc(cfg(feature = "cl1_2")))]
pub struct FillBuffer<Dst> {
    event: RawEvent,
    dst: Dst
}

impl<T: Copy + Unpin, Dst: DerefMut<Target = Buffer<T, C>>, C: Context> FillBuffer<Dst> {
    #[inline(always)]
    pub unsafe fn new (src: T, mut dst: Dst, range: impl IntoRange, queue: &RawCommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let event = (&mut dst as &mut RawBuffer).fill_raw(src, range, queue, wait)?;
        Ok(Self { event, dst })
    }
}

impl<T: Copy + Unpin, Dst: DerefMut<Target = Buffer<T, C>>, C: Context> Event for FillBuffer<Dst> {
    type Output = Dst;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) }
        Ok(self.dst)
    }
}