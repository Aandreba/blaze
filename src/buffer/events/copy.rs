use std::{ops::{Deref, DerefMut}};
use crate::{prelude::*, buffer::{Buffer, RawBuffer}, event::WaitList};

pub struct CopyBuffer<Src, Dst> {
    event: RawEvent,
    src: Src,
    dst: Dst
}

impl<T: Copy + Unpin, Src: Deref<Target = Buffer<T, C>>, Dst: DerefMut<Target = Buffer<T, C>>, C: Context> CopyBuffer<Src, Dst> {
    #[inline]
    pub unsafe fn new (src: Src, offset_src: usize, mut dst: Dst, offset_dst: usize, len: usize, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<Self> {
        let dst_offset = offset_dst.checked_mul(core::mem::size_of::<T>()).unwrap();
        let src_offset = offset_src.checked_mul(core::mem::size_of::<T>()).unwrap();
        let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();

        let event = (&mut dst as &mut RawBuffer).copy_from(dst_offset, &src, src_offset, size, &queue, wait)?;
        Ok(Self { event, src, dst })
    }   
}

impl<T: Copy + Unpin, Src: Deref<Target = Buffer<T, C>>, Dst: DerefMut<Target = Buffer<T, C>>, C: Context> Event for CopyBuffer<Src, Dst> {
    type Output = (Src, Dst);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) };
        Ok((self.src, self.dst))
    }
}