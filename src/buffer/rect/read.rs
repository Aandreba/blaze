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
        src: &'src RawBuffer, max_rows: usize, max_cols: usize, slice: impl IntoSlice2D,
        buffer_row_pitch: Option<usize>, queue: &CommandQueue, wait: impl Into<WaitList>
    ) -> Result<Self> {
        if let Some(slice) = slice.into_slice(max_rows, max_cols) {
            let [offset, region] = slice.raw_parts_buffer::<T>();
            
            let mut dst = Rect2D::<T>::new_uninit(slice.width(), slice.height()).unwrap();
            let event = src.read_rect_to_ptr(offset, [0, 0, 0], region, buffer_row_pitch, Some(0), Some(0), Some(0), dst.as_mut_ptr() as *mut T, queue, wait)?;
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