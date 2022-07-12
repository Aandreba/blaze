flat_mod!(view);
use std::ptr::NonNull;
use image::ImageBuffer;
use crate::{prelude::*, memobj::{MapBox, Map, MapRefBox, MapMutBox, MapMut}, image::{RawImage, IntoSlice, channel::RawPixel}, event::WaitList};

pub struct MapImage2D<P: RawPixel, C: Context> {
    event: RawEvent,
    mem: RawImage,
    width: usize,
    height: usize,
    ptr: NonNull<P::Subpixel>,
    ctx: C
}

impl<P: RawPixel, C: Context> MapImage2D<P, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: RawImage, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<Self> {
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

impl<P: RawPixel, C: Context> Event for MapImage2D<P, C> {
    type Output = ImageBuffer<P, MapBox<P::Subpixel, C>>;

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
            buf = MapBox::from_raw_in(ptr, Map::new_in(self.ctx, self.mem.into()));
        }

        Ok(ImageBuffer::from_raw(u32::try_from(self.width).unwrap(), u32::try_from(self.height).unwrap(), buf).unwrap())
    }
}

pub struct MapRefImage2D<'a, P: RawPixel, C: Context> {
    event: RawEvent,
    mem: &'a RawImage,
    width: usize,
    height: usize,
    ptr: NonNull<P::Subpixel>,
    ctx: C
}

impl<'a, P: RawPixel, C: Context> MapRefImage2D<'a, P, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: &'a RawImage, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<Self> {
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

impl<'a, P: RawPixel, C: Context> Event for MapRefImage2D<'a, P, C> {
    type Output = ImageBuffer<P, MapRefBox<'a, P::Subpixel, C>>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.event
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = err { return Err(err); }
        let len = self.height.checked_mul(self.width).and_then(|x| x.checked_mul(P::CHANNEL_COUNT as usize)).unwrap();

        let buf = unsafe {
            MapRefBox::from_raw_parts_in(self.mem, self.ptr.as_ptr(), len, self.ctx)
        };

        Ok(ImageBuffer::from_raw(u32::try_from(self.width).unwrap(), u32::try_from(self.height).unwrap(), buf).unwrap())
    }
}

pub struct MapMutImage2D<'a, P: RawPixel, C: Context> {
    event: RawEvent,
    mem: &'a mut RawImage,
    width: usize,
    height: usize,
    ptr: NonNull<P::Subpixel>,
    ctx: C
}

impl<'a, P: RawPixel, C: Context> MapMutImage2D<'a, P, C> {
    #[inline(always)]
    pub unsafe fn new (ctx: C, src: &'a mut RawImage, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<Self> {
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

impl<'a, P: RawPixel, C: Context> Event for MapMutImage2D<'a, P, C> {
    type Output = ImageBuffer<P, MapMutBox<'a, P::Subpixel, C>>;

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