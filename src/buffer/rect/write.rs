use std::{marker::PhantomData, pin::Pin, ops::Deref};
use crate::{buffer::RawBuffer, prelude::*, event::WaitList};
use super::Rect2D;

pub struct WriteBufferRect2D<'dst, D> {
    event: RawEvent,
    #[allow(unused)]
    src: Pin<D>,
    dst: PhantomData<&'dst mut RawBuffer>
}

impl<'dst, D: Deref<Target = Rect2D<T>>, T: Copy + Unpin> WriteBufferRect2D<'dst, D> {
    #[inline]
    pub unsafe fn new (
        src: D, offset_src: [usize; 2], dst: &'dst mut RawBuffer, offset_dst: [usize; 2], region: [usize; 2],
        buffer_row_pitch: Option<usize>, buffer_slice_pitch: Option<usize>, queue: &CommandQueue, wait: impl Into<WaitList>
    ) -> Result<Self> {

        let src = Pin::new(src);
        let host_row_pitch = src.width() * core::mem::size_of::<T>();
        let buffer_origin = [offset_src[0] * core::mem::size_of::<T>(), offset_src[1], 0];
        let host_origin = [offset_dst[0] * core::mem::size_of::<T>(), offset_dst[1], 0];
        let region = [region[0] * core::mem::size_of::<T>(), region[1], 1];

        let event = dst.write_rect_from_ptr(buffer_origin, host_origin, region, buffer_row_pitch, buffer_slice_pitch, Some(host_row_pitch), Some(0), src.as_ptr(), queue, wait)?;
        return Ok(Self { event, dst: PhantomData, src })
    }
}

impl<'dst, D: Deref<Target = Rect2D<T>>, T: Copy + Unpin> Event for WriteBufferRect2D<'dst, D> {
    type Output = ();

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        Ok(())
    }
}