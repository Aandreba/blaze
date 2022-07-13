use std::{marker::PhantomData, mem::MaybeUninit};
use crate::{buffer::RawBuffer, prelude::*, event::WaitList, image::{IntoSlice, ImageSlice}};
use super::Rect2D;

pub struct ReadBufferRect2D<'src, T> {
    event: RawEvent,
    dst: Rect2D<MaybeUninit<T>>,
    src: PhantomData<&'src RawBuffer>
}

impl<'src, T: Copy + Unpin> ReadBufferRect2D<'src, T> {
    #[inline]
    pub unsafe fn new (
        src: &'src RawBuffer, max_rows: usize, max_cols: usize, slice: impl IntoSlice<2>, 
        buffer_row_pitch: Option<usize>, buffer_slice_pitch: Option<usize>, queue: &CommandQueue, wait: impl Into<WaitList>
    ) -> Result<Self> {

        let slice : ImageSlice = slice.into_slice([max_rows, max_cols]);
        let (offset, region) = slice.scaled_parts::<T>();
        let host_row_pitch = region[0].checked_mul(core::mem::size_of::<T>()).unwrap();
        let host_slice_pitch = region[1].checked_mul(host_row_pitch).unwrap();

        let mut dst = Rect2D::<T>::new_uninit(region[0], region[1]).unwrap();
        let event = src.read_rect_to_ptr(offset, core::mem::zeroed(), region, buffer_row_pitch, buffer_slice_pitch, Some(host_row_pitch), Some(host_slice_pitch), dst.as_mut_ptr() as *mut T, queue, wait)?;
        Ok(Self { event, dst, src: PhantomData })
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