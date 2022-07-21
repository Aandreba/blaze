use std::{mem::MaybeUninit, marker::PhantomData, num::NonZeroUsize};
use crate::{prelude::*, image::{channel::RawPixel, RawImage}, event::WaitList, memobj::{IntoSlice2D}, buffer::rect::Rect2D};

pub struct ReadImage2D<'src, P: RawPixel> {
    event: RawEvent,
    width: usize,
    height: usize,
    dst: Rect2D<MaybeUninit<P>>,
    src: PhantomData<&'src P>
}

impl<'src, P: RawPixel + Unpin> ReadImage2D<'src, P> {
    #[inline]
    pub unsafe fn new (src: &'src RawImage, queue: &CommandQueue, slice: impl IntoSlice2D, row_pitch: Option<usize>, slice_pitch: Option<usize>, wait: impl Into<WaitList>) -> Result<Self> {
        if let Some(slice) = slice.into_slice(src.width()?, src.height()?) {
            let [origin, region] = slice.raw_parts();
            let size = slice.size().and_then(|x| x.checked_mul(NonZeroUsize::new(P::CHANNEL_COUNT as usize).unwrap())).unwrap().get();

            let mut result = Rect2D::new_uninit(src.width()?, src.height()?).unwrap();
            let event = src.read_to_ptr(origin, region, row_pitch, slice_pitch, result.as_mut_ptr().cast(), queue, wait)?;

            return Ok(Self {
                event,
                width: slice.width(),
                height: slice.height(),
                dst: result,
                src: PhantomData
            })
        }

        todo!()
    }
}

impl<'src, P: RawPixel + Unpin> Event for ReadImage2D<'src, P> {
    type Output = Rect2D<P>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) };
        let pixels = unsafe { self.dst.assume_init() };
        Ok(pixels)
    }
}