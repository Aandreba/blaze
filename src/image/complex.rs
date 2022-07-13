use std::{ptr::NonNull, os::raw::c_void, marker::PhantomData, ops::{Deref, DerefMut}, path::Path, io::{Seek, BufRead}, borrow::Borrow};
use image::{ImageBuffer, io::Reader};
use rscl_proc::docfg;
use crate::{core::*, context::{Context, Global}, buffer::{flags::{HostPtr, MemFlags, MemAccess}}, event::WaitList, memobj::MemObjectType};
use super::{RawImage, ImageDesc, channel::{RawPixel, FromDynamic, FromPrimitive}, IntoSlice, events::{ReadImage2D, WriteImage2D, CopyImage, FillImage}};

#[derive(Debug)]
pub struct Image2D<P: RawPixel, C: Context = Global> {
    inner: RawImage,
    ctx: C,
    phtm: PhantomData<P>
}

impl<P: RawPixel> Image2D<P> {
    #[inline(always)]
    pub fn from_file (path: impl AsRef<Path>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynamic {
        Self::from_file_in(Global, path, access, alloc)
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
    pub unsafe fn create (width: usize, height: usize, flags: impl Into<MemFlags>, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        Self::create_in(Global, width, height, flags, host_ptr)
    }
}

impl<P: RawPixel, C: Context> Image2D<P, C> {
    #[inline]
    pub fn from_file_in (ctx: C, path: impl AsRef<Path>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynamic {
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
        let host = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, width, height, host, NonNull::new(v as *const _ as *mut _)) }
    }

    #[inline(always)]
    pub unsafe fn uninit_in (ctx: C, width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        let host = MemFlags::new(access, HostPtr::new(alloc, false));
        Self::create_in(ctx, width, height, host, None)
    }
    
    #[inline]
    pub unsafe fn create_in (ctx: C, width: usize, height: usize, flags: impl Into<MemFlags>, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        let flags = flags.into();
        let desc = ImageDesc::new(MemObjectType::Image2D, width, height);

        #[cfg(feature = "cl1_2")]
        let inner = RawImage::new(ctx.as_raw(), flags, P::FORMAT, desc, host_ptr)?;
        #[cfg(not(feature = "cl1_2"))]
        let inner = RawImage::new_2d(ctx.raw_context(), flags, P::FORMAT, desc, host_ptr)?;

        Ok(Self { inner, ctx, phtm: PhantomData })
    }

    /// Returns a reference to the image's [`RawImage`].
    #[inline(always)]
    pub fn as_raw (&self) -> &RawImage {
        &self.inner
    }

    /// Returns a reference to the image's [`Context`].
    #[inline(always)]
    pub fn context (&self) -> &C {
        &self.ctx
    }
}

impl<P: RawPixel, C: Context> Image2D<P, C> where P::Subpixel: Unpin {
    #[inline(always)]
    pub fn read_all (&self, wait: impl Into<WaitList>) -> Result<ReadImage2D<P>> {
        self.read((.., ..), wait)
    }

    #[inline(always)]
    pub fn read (&self, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<ReadImage2D<P>> {
        self.read_with_pitch(slice, None, None, wait)
    }

    #[inline(always)]
    pub fn read_with_pitch (&self, slice: impl IntoSlice<2>, row_pitch: Option<usize>, slice_pitch: Option<usize>, wait: impl Into<WaitList>) -> Result<ReadImage2D<P>> {
        unsafe { ReadImage2D::new(self, self.context().next_queue(), slice, row_pitch, slice_pitch, wait) }
    }

    #[inline(always)]
    pub fn write<'src, 'dst, Raw: Deref<Target = [P::Subpixel]>> (&'dst mut self, src: &'src ImageBuffer<P, Raw>, offset: [usize; 2], wait: impl Into<WaitList>) -> Result<WriteImage2D<'src, 'dst, P>> {
        self.write_with_pitch(src, offset, None, None, wait)
    }

    #[inline(always)]
    pub fn write_with_pitch<'src, 'dst, Raw: Deref<Target = [P::Subpixel]>> (&'dst mut self, src: &'src ImageBuffer<P, Raw>, offset: [usize; 2], row_pitch: Option<usize>, slice_pitch: Option<usize>, wait: impl Into<WaitList>) -> Result<WriteImage2D<'src, 'dst, P>> {
        unsafe { WriteImage2D::new(src, &mut self.inner, self.ctx.next_queue(), offset, row_pitch, slice_pitch, wait) }
    }

    #[inline(always)]
    pub fn copy_from<'src, 'dst> (&'dst mut self, offset_dst: [usize; 2], src: &'src Self, offset_src: [usize; 2], region: [usize; 2], wait: impl Into<WaitList>) -> Result<CopyImage<'src, 'dst>> {
        unsafe { CopyImage::new(src, offset_src, &mut self.inner, offset_dst, region, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub fn copy_to<'src, 'dst> (&'src self, offset_src: [usize; 2], dst: &'dst mut Self, offset_dst: [usize; 2], region: [usize; 2], wait: impl Into<WaitList>) -> Result<CopyImage<'src, 'dst>> {
        unsafe { CopyImage::new(&self.inner, offset_src, dst, offset_dst, region, self.ctx.next_queue(), wait) }
    }

    #[inline]
    pub fn fill<'dst> (&'dst mut self, color: impl Borrow<P>, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<FillImage<'dst>> where f32: FromPrimitive<P::Subpixel> {
        unsafe { FillImage::new(&mut self.inner, color.borrow(), slice, self.ctx.next_queue(), wait) }
    }

    /*#[docfg(feature = "map")]
    #[inline(always)]
    pub fn map<'a> (&'a self, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<super::events::MapImage2D<P, &'a Self, C>> where P: 'static, C: 'static + Clone {
        Self::map_by_deref(self, slice, wait)
    }

    #[docfg(feature = "map")]
    #[inline(always)]
    pub fn map_mut<'a> (&'a mut self, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<super::events::MapMutImage2D<P, &'a mut Self, C>> where P: 'static, C: 'static + Clone {
        Self::map_by_deref_mut(self, slice, wait)
    }

    #[docfg(feature = "map")]
    #[inline(always)]
    pub fn map_by_deref<D: Deref<Target = Self>> (this: D, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<super::events::MapImage2D<P, D, C>> where P: 'static, C: 'static + Clone {
        unsafe { super::events::MapImage2D::new(this.ctx.clone(), this, slice, wait) }
    }

    #[docfg(feature = "map")]
    #[inline(always)]
    pub fn map_by_deref_mut<D: DerefMut<Target = Self>> (this: D, slice: impl IntoSlice<2>, wait: impl Into<WaitList>) -> Result<super::events::MapMutImage2D<P, D, C>> where P: 'static, C: 'static + Clone {
        unsafe { super::events::MapMutImage2D::new(this.ctx.clone(), this, slice, wait) }
    }*/
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