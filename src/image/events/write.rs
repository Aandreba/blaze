use std::{marker::PhantomData};
use crate::{prelude::*, image::{channel::RawPixel, RawImage}, event::WaitList, memobj::{Slice2D}, buffer::rect::Rect2D};

#[repr(transparent)]
pub struct WriteImage2D<'src, 'dst, P: RawPixel> where P::Subpixel: Unpin {
    event: RawEvent,
    src: PhantomData<&'src P>,
    dst: PhantomData<&'dst mut P>
}

impl<'src, 'dst, P: RawPixel> WriteImage2D<'src, 'dst, P> where P::Subpixel: Unpin {
    #[inline]
    pub unsafe fn new (src: &'src Rect2D<P>, dst: &'dst mut RawImage, queue: &CommandQueue, offset: [usize; 2], row_pitch: Option<usize>, slice_pitch: Option<usize>, wait: impl Into<WaitList>) -> Result<Self> {
        if let Some(slice) = Slice2D::try_new(offset[0], offset[1], src.width() as usize, src.height() as usize) {
            let [origin, region] = slice.raw_parts();
            let event = dst.write_from_ptr(origin, region, row_pitch, slice_pitch, src.as_ptr().cast(), queue, wait)?;

            return Ok(Self {
                event,
                src: PhantomData,
                dst: PhantomData
            })
        }

        todo!()
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