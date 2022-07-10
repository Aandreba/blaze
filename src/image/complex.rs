use std::{ptr::NonNull, os::raw::c_void, marker::PhantomData, ops::{Deref, DerefMut}};
use bytemuck::{NoUninit, cast_slice};
use image::{ImageBuffer};

use crate::{core::*, context::{Context, Global}, buffer::flags::{MemAccess, HostPtr, FullMemFlags}};
use super::{RawImage, ImageDesc, channel::RawPixel};

#[derive(Debug, Clone)]
pub struct Image2D<P: RawPixel, C: Context = Global> {
    inner: RawImage,
    ctx: C,
    phtm: PhantomData<P>
}

impl<P: RawPixel> Image2D<P> {
    #[inline(always)]
    pub fn new<Raw: Deref<Target = [P::Subpixel]>> (image: &ImageBuffer<P, Raw>, alloc: bool) -> Result<Self> where P::Subpixel: NoUninit {
        Self::new_in(Global, image, alloc)
    }

    #[inline(always)]
    pub fn new_raw (v: &[u8], width: usize, height: usize, alloc: bool) -> Result<Self> {
        Self::new_raw_in(Global, v, width, height, alloc)
    }

    #[inline(always)]
    pub unsafe fn uninit (width: usize, height: usize, alloc: bool) -> Result<Self> {
        Self::uninit_in(Global, width, height, alloc)
    }

    #[inline(always)]
    pub unsafe fn create (width: usize, height: usize, host: HostPtr, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        Self::create_in(Global, width, height, host, host_ptr)
    }
}

impl<P: RawPixel, C: Context> Image2D<P, C> {
    pub fn new_in<Raw: Deref<Target = [P::Subpixel]>> (ctx: C, image: &ImageBuffer<P, Raw>, alloc: bool) -> Result<Self> where P::Subpixel: NoUninit {
        let width = image.width() as usize;
        let height = image.height() as usize;

        let v : &[u8] = cast_slice(image.as_raw().deref());
        Self::new_raw_in(ctx, v, width, height, alloc)
    }

    #[inline(always)]
    pub fn new_raw_in (ctx: C, v: &[u8], width: usize, height: usize, alloc: bool) -> Result<Self> {
        let host = HostPtr::new(alloc, true);
        unsafe { Self::create_in(ctx, width, height, host, NonNull::new(v as *const _ as *mut _)) }
    }

    #[inline(always)]
    pub unsafe fn uninit_in (ctx: C, width: usize, height: usize, alloc: bool) -> Result<Self> {
        let host = HostPtr::new(alloc, false);
        Self::create_in(ctx, width, height, host, None)
    }
    
    pub unsafe fn create_in (ctx: C, width: usize, height: usize, host: HostPtr, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        let flags = FullMemFlags::new(MemAccess::READ_WRITE, host);
        let desc = ImageDesc::new(MemObjectType::Image2D, width, height);

        #[cfg(feature = "cl1_2")]
        let inner = RawImage::new(ctx.raw_context(), flags, P::FORMAT, desc, host_ptr)?;
        #[cfg(not(feature = "cl1_2"))]
        let inner = RawImage::new_2d(ctx.raw_context(), flags, P::FORMAT, desc, host_ptr)?;

        Ok(Self { inner, ctx, phtm: PhantomData })
    }
}

impl<P: RawPixel, C: Context> Deref for Image2D<P, C> {
    type Target = RawImage;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: RawPixel, C: Context> DerefMut for Image2D<P, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}