use std::{pin::Pin, ops::Deref, marker::PhantomData};
use image::ImageBuffer;
use crate::{prelude::*, image::{channel::RawPixel, RawImage, ImageSlice}, event::WaitList};

#[repr(transparent)]
pub struct WriteImage2D<'src, 'dst, P: RawPixel> where P::Subpixel: Unpin {
    event: RawEvent,
    src: PhantomData<&'src P>,
    dst: PhantomData<&'dst mut P>
}

impl<'src, 'dst, P: RawPixel> WriteImage2D<'src, 'dst, P> where P::Subpixel: Unpin {
    #[inline]
    pub unsafe fn new<C: Deref<Target = [P::Subpixel]>> (src: &'src ImageBuffer<P, C>, dst: &'dst mut RawImage, queue: &CommandQueue, offset: [usize; 2], row_pitch: Option<usize>, slice_pitch: Option<usize>, wait: impl Into<WaitList>) -> Result<Self> {
        let slice = ImageSlice::new_2d(offset, [src.width() as usize, src.height() as usize]);
        let buffer = Pin::new(src.as_raw().deref());

        let event = dst.write_from_ptr(slice, row_pitch, slice_pitch, buffer.as_ptr().cast(), queue, wait)?;
        Ok(Self {
            event,
            src: PhantomData,
            dst: PhantomData
        })
    }
}

impl<'src, 'dst, P: RawPixel> Event for WriteImage2D<'src, 'dst, P> where P::Subpixel: Unpin {
    type Output = ();

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err) }
        Ok(())
    }
}