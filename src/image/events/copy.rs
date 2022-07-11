use std::marker::PhantomData;
use crate::{image::{RawImage}, prelude::*, event::WaitList};

#[repr(transparent)]
pub struct CopyImage<'src, 'dst> {
    event: RawEvent,
    src: PhantomData<&'src RawImage>,
    dst: PhantomData<&'dst mut RawImage>
}

impl<'src, 'dst> CopyImage<'src, 'dst> {
    pub unsafe fn new<const N: usize> (src: &'src RawImage, offset_src: [usize;N], dst: &'dst mut RawImage, offset_dst: [usize;N], region: [usize;N], queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let event = dst.copy_from(offset_dst, src, offset_src, region, queue, wait)?;
        Ok(Self {
            event,
            src: PhantomData,
            dst: PhantomData
        })
    }
}

impl<'src, 'dst> Event for CopyImage<'src, 'dst> {
    type Output = ();

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &&self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) };
        Ok(())
    }
}