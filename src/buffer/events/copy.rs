use std::marker::PhantomData;
use crate::{prelude::*, buffer::RawBuffer, event::WaitList};

#[repr(transparent)]
pub struct CopyBuffer<'src, 'dst> {
    event: RawEvent,
    src: PhantomData<&'src RawBuffer>,
    dst: PhantomData<&'dst mut RawBuffer>
}

impl<'src, 'dst> CopyBuffer<'src, 'dst> {
    #[inline]
    pub unsafe fn new<T: Copy, W: Into<WaitList>> (src: &RawBuffer, offset_src: usize, dst: &mut RawBuffer, offset_dst: usize, len: usize, queue: &CommandQueue, wait: W) -> Result<Self> {
        let dst_offset = offset_dst.checked_mul(core::mem::size_of::<T>()).unwrap();
        let src_offset = offset_src.checked_mul(core::mem::size_of::<T>()).unwrap();
        let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();

        let event = dst.copy_from(dst_offset, src, src_offset, size, &queue, wait)?;
        Ok(Self { event, src: PhantomData, dst: PhantomData })
    }   
}

impl<'src, 'dst> Event for CopyBuffer<'src, 'dst> {
    type Output = ();

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) };
        Ok(())
    }
}