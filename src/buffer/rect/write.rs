use std::{pin::Pin, ops::{Deref, DerefMut}};
use crate::{prelude::*, event::WaitList};
use super::{Rect2D, BufferRect2D};

#[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
pub struct WriteBufferRect2D<Dst, Src> {
    event: RawEvent,
    src: Pin<Dst>,
    dst: Src
}

impl<T: Copy + Unpin, Src: Deref<Target = Rect2D<T>>, Dst: DerefMut<Target = BufferRect2D<T, C>>, C: Context> WriteBufferRect2D<Src, Dst> {
    #[inline]
    pub unsafe fn new (
        src: Src, offset_src: [usize; 2], mut dst: Dst, offset_dst: [usize; 2], region: [usize; 2],
        buffer_row_pitch: Option<usize>, buffer_slice_pitch: Option<usize>, queue: &RawCommandQueue, wait: impl Into<WaitList>
    ) -> Result<Self> {

        let src = Pin::new(src);
        let host_row_pitch = src.width() * core::mem::size_of::<T>();
        let buffer_origin = [offset_src[0] * core::mem::size_of::<T>(), offset_src[1], 0];
        let host_origin = [offset_dst[0] * core::mem::size_of::<T>(), offset_dst[1], 0];
        let region = [region[0] * core::mem::size_of::<T>(), region[1], 1];

        let event = dst.write_rect_from_ptr_in(buffer_origin, host_origin, region, buffer_row_pitch, buffer_slice_pitch, Some(host_row_pitch), Some(0), src.as_ptr(), queue, wait)?;
        return Ok(Self { event, dst, src })
    }
}

impl<T: Copy + Unpin, Src: Deref<Target = Rect2D<T>>, Dst: DerefMut<Target = BufferRect2D<T, C>>, C: Context> Event for WriteBufferRect2D<Src, Dst> {
    type Output = (Src, Dst);

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        Ok((Pin::into_inner(self.src), self.dst))
    }
}