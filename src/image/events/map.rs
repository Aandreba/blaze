use std::ptr::NonNull;
use image::ImageBuffer;
use crate::{prelude::*, memobj::{MapMutBox, MapMut, MapBox, AsMem, AsMutMem}, image::{IntoSlice, channel::RawPixel}, event::WaitList};

pub struct MapImage2D<P: RawPixel, D, C: Context> {
    event: RawEvent,
    mem: D,
    width: usize,
    height: usize,
    ptr: NonNull<P::Subpixel>,
    ctx: C
}

impl<P: RawPixel, D: AsMem, C: Context> MapImage2D<P, D, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: D, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<Self> {
        let range = slice.into_slice([src.width()?, src.height()?]);
        let (ptr, _, _, event) = src.map_read_write(range, ctx.next_queue(), wait)?;
        let ptr : NonNull<P::Subpixel> = NonNull::new(ptr).unwrap();

        Ok(Self { 
            event, ptr, ctx,
            width: range.width(),
            height: range.height(),
            mem: src
        })
    }
}

impl<P: RawPixel, D: AsMem, C: Context> Event for MapImage2D<P, D, C> {
    type Output = ImageBuffer<P, MapBox<P::Subpixel, D, C>>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        let len = self.height.checked_mul(self.width).and_then(|x| x.checked_mul(P::CHANNEL_COUNT as usize)).unwrap();

        let buf = unsafe {
            MapBox::from_raw_parts_in(self.mem, self.ptr.as_ptr(), len, self.ctx)
        };

        Ok(ImageBuffer::from_raw(u32::try_from(self.width).unwrap(), u32::try_from(self.height).unwrap(), buf).unwrap())
    }
}

pub struct MapMutImage2D<P: RawPixel, D: AsMutMem, C: Context> {
    event: RawEvent,
    mem: D,
    width: usize,
    height: usize,
    ptr: NonNull<P::Subpixel>,
    ctx: C
}

impl<P: RawPixel, D: AsMutMem, C: Context> MapMutImage2D<P, D, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: D, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<Self> {
        let range = slice.into_slice([src.width()?, src.height()?]);
        let (ptr, _, _, event) = src.map_read_write(range, ctx.next_queue(), wait)?;
        let ptr : NonNull<P::Subpixel> = NonNull::new(ptr).unwrap();

        Ok(Self { 
            event, ptr, ctx,
            width: range.width(),
            height: range.height(),
            mem: src
        })
    }
}

impl<P: RawPixel, D: AsMutMem, C: Context> Event for MapMutImage2D<P, D, C> {
    type Output = ImageBuffer<P, MapMutBox<P::Subpixel, D, C>>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }

        let buf;
        unsafe {
            let len = self.height.checked_mul(self.width).and_then(|x| x.checked_mul(P::CHANNEL_COUNT as usize)).unwrap();
            let ptr = core::slice::from_raw_parts_mut(self.ptr.as_ptr(), len);
            buf = MapMutBox::from_raw_in(ptr, MapMut::new_in(self.ctx, self.mem));
        }

        Ok(ImageBuffer::from_raw(u32::try_from(self.width).unwrap(), u32::try_from(self.height).unwrap(), buf).unwrap())
    }
}