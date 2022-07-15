use std::{marker::PhantomData, mem::MaybeUninit};
use crate::{buffer::RawBuffer, prelude::*, event::WaitList, memobj::{IntoSlice2D}};
use super::Rect2D;

pub struct ReadBufferRect2D<'src, T> {
    event: RawEvent,
    dst: Rect2D<MaybeUninit<T>>,
    src: PhantomData<&'src RawBuffer>
}

impl<'src, T: Copy + Unpin> ReadBufferRect2D<'src, T> {
    #[inline]
    pub unsafe fn new (
        src: &'src RawBuffer, max_width: usize, max_height: usize, slice: impl IntoSlice2D,
        buffer_row_pitch: Option<usize>, buffer_slice_pitch: Option<usize>, queue: &CommandQueue, wait: impl Into<WaitList>
    ) -> Result<Self> {

        if let Some(slice) = slice.into_slice(max_width, max_height) {
            let [buffer_origin, region] = slice.raw_parts_buffer::<T>();
            let mut dst = Rect2D::<T>::new_uninit(slice.width(), slice.height()).unwrap();
            let event = src.read_rect_to_ptr(buffer_origin, [0; 3], region, buffer_row_pitch, buffer_slice_pitch, Some(0), Some(0), dst.as_mut_ptr() as *mut T, queue, wait)?;
            return Ok(Self { event, dst, src: PhantomData })
        }

        Err(Error::new(ErrorType::InvalidBufferSize, "error calculating buffer size (possible arithmetic overflow)"))
    }
}

impl<'src, T: Copy + Unpin> Event for ReadBufferRect2D<'src, T> {
    type Output = Rect2D<T>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        unsafe { Ok(self.dst.assume_init()) }
    }
}

pub struct ReadIntoBufferRect2D<'src, 'dst> {
    event: RawEvent,
    dst: PhantomData<&'dst mut ()>,
    src: PhantomData<&'src RawBuffer>
}

impl<'src, 'dst> ReadIntoBufferRect2D<'src, 'dst> {
    #[inline]
    pub unsafe fn new<T: Copy + Unpin> (
        src: &'src RawBuffer, offset_src: [usize; 2], dst: &'dst mut Rect2D<T>, offset_dst: [usize; 2], region: [usize; 2],
        buffer_row_pitch: Option<usize>, buffer_slice_pitch: Option<usize>, queue: &CommandQueue, wait: impl Into<WaitList>
    ) -> Result<Self> {

        let buffer_origin = [offset_src[0] * core::mem::size_of::<T>(), offset_src[1], 0];
        let host_origin = [offset_dst[0] * core::mem::size_of::<T>(), offset_dst[1], 0];
        let region = [region[0] * core::mem::size_of::<T>(), region[1], 1];

        let event = src.read_rect_to_ptr(buffer_origin, host_origin, region, buffer_row_pitch, buffer_slice_pitch, Some(0), Some(0), dst.as_mut_ptr(), queue, wait)?;
        return Ok(Self { event, dst: PhantomData, src: PhantomData })
    }
}

impl<'src, 'dst> Event for ReadIntoBufferRect2D<'src, 'dst> {
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