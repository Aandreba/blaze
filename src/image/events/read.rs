use std::{pin::Pin, mem::MaybeUninit, marker::PhantomData};
use image::ImageBuffer;
use crate::{prelude::*, image::{channel::RawPixel, IntoSlice, RawImage}, event::WaitList};

pub struct ReadImage2D<'src, P: RawPixel> {
    event: RawEvent,
    width: usize,
    height: usize,
    dst: Pin<Box<[MaybeUninit<P::Subpixel>]>>,
    src: PhantomData<&'src P>
}

impl<'src, P: RawPixel> ReadImage2D<'src, P> where P::Subpixel: Unpin {
    #[inline]
    pub unsafe fn new (src: &'src RawImage, queue: &CommandQueue, slice: impl IntoSlice<2>, row_pitch: Option<usize>, slice_pitch: Option<usize>, wait: impl Into<WaitList>) -> Result<Self> {
        let slice = slice.into_slice([src.width()?, src.height()?]);
        let size = slice.size().checked_mul(P::CHANNEL_COUNT as usize).unwrap();

        let mut result = Pin::new(Box::new_uninit_slice(size));
        let event = src.read_to_ptr(slice, row_pitch, slice_pitch, result.as_mut_ptr().cast(), queue, wait)?;

        Ok(Self {
            event,
            width: slice.width(),
            height: slice.height(),
            dst: result,
            src: PhantomData
        })
    }
}

impl<'src, P: RawPixel> Event for ReadImage2D<'src, P> where P::Subpixel: Unpin {
    type Output = ImageBuffer<P, Box<[P::Subpixel]>>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) };
        let pixels = unsafe { Pin::into_inner(self.dst).assume_init() };

        let width = u32::try_from(self.width).unwrap();
        let height = u32::try_from(self.height).unwrap();

        Ok(ImageBuffer::from_raw(width, height, pixels).unwrap())
    }
}