use std::{ptr::NonNull, os::raw::c_void, marker::PhantomData, ops::{Deref, DerefMut}, path::Path, io::{Seek, BufRead}};
use image::{ImageBuffer, io::Reader};
use crate::{core::*, context::{Context, Global}, buffer::flags::{HostPtr, FullMemFlags, MemAccess}};
use super::{RawImage, ImageDesc, channel::{RawPixel, FromDynamic}};

#[derive(Debug, Clone)]
pub struct Image2D<P: RawPixel, C: Context = Global> {
    inner: RawImage,
    ctx: C,
    phtm: PhantomData<P>
}

impl<P: RawPixel> Image2D<P> {
    #[inline(always)]
    pub fn read (path: impl AsRef<Path>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynamic {
        Self::read_in(Global, path, access, alloc)
    }

    #[inline(always)]
    pub fn from_reader<R: BufRead + Seek> (reader: Reader<R>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynamic {
        Self::from_reader_in(Global, reader, access, alloc)
    }

    #[inline(always)]
    pub fn from_buffer<Raw: Deref<Target = [P::Subpixel]>> (buffer: &ImageBuffer<P, Raw>, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::from_buffer_in(Global, buffer, access, alloc)
    }

    #[inline(always)]
    pub fn from_raw (v: &[P::Subpixel], width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::from_raw_in(Global, v, width, height, access, alloc)
    }

    #[inline(always)]
    pub unsafe fn uninit (width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::uninit_in(Global, width, height, access, alloc)
    }

    #[inline(always)]
    pub unsafe fn create (width: usize, height: usize, flags: impl Into<FullMemFlags>, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        Self::create_in(Global, width, height, flags, host_ptr)
    }
}

impl<P: RawPixel, C: Context> Image2D<P, C> {
    #[inline]
    pub fn read_in (ctx: C, path: impl AsRef<Path>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynamic {
        let reader = match Reader::open(path) {
            Ok(x) => x,
            Err(e) => return Err(Error::new(ErrorType::InvalidValue, e))
        };

        Self::from_reader_in(ctx, reader, access, alloc)
    }

    /// Creates a new 2D image from an image reader.
    #[inline]
    pub fn from_reader_in<R: BufRead + Seek> (ctx: C, reader: Reader<R>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynamic {
        let decode = match reader.decode() {
            Ok(x) => x,
            Err(e) => return Err(Error::new(ErrorType::InvalidValue, e))
        };

        let buffer = P::from_dynamic(decode);
        Self::from_buffer_in(ctx, &buffer, access, alloc)
    }

    /// Creates a new 2D image from an image buffer.
    #[inline]
    pub fn from_buffer_in<Raw: Deref<Target = [P::Subpixel]>> (ctx: C, buffer: &ImageBuffer<P, Raw>, access: MemAccess, alloc: bool) -> Result<Self> {
        let width = buffer.width() as usize;
        let height = buffer.height() as usize;

        let v : &[P::Subpixel] = buffer.as_raw().deref();
        Self::from_raw_in(ctx, v, width, height, access, alloc)
    }
    
    /// Creates a new 2D image from it's raw pixels.
    #[inline(always)]
    pub fn from_raw_in (ctx: C, v: &[P::Subpixel], width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        let host = FullMemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, width, height, host, NonNull::new(v as *const _ as *mut _)) }
    }

    #[inline(always)]
    pub unsafe fn uninit_in (ctx: C, width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        let host = FullMemFlags::new(access, HostPtr::new(alloc, false));
        Self::create_in(ctx, width, height, host, None)
    }
    
    #[inline]
    pub unsafe fn create_in (ctx: C, width: usize, height: usize, flags: impl Into<FullMemFlags>, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        let flags = flags.into();
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