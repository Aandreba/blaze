use std::{ptr::NonNull, os::raw::c_void, marker::PhantomData, ops::{Deref, DerefMut}};
use opencl_sys::cl_mem;
use crate::{core::*, context::{Context, Global}, buffer::{flags::{HostPtr, MemFlags, MemAccess}, rect::Rect2D}, event::WaitList, memobj::{MemObjectType, IntoSlice2D, RawMemObject}};
use super::{RawImage, ImageDesc, channel::{RawPixel}, events::{ReadImage2D, WriteImage2D, CopyImage}};

#[derive(Debug)]
pub struct Image2D<P: RawPixel, C: Context = Global> {
    inner: RawImage,
    ctx: C,
    phtm: PhantomData<P>
}

impl<P: RawPixel> Image2D<P> {
    /*#[inline(always)]
    pub fn from_file (path: impl AsRef<Path>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynImage {
        Self::from_file_in(Global, path, access, alloc)
    }

    #[inline(always)]
    pub fn from_reader<R: BufRead + Seek> (reader: Reader<R>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynImage {
        Self::from_reader_in(Global, reader, access, alloc)
    }*/

    #[inline(always)]
    pub fn from_rect (v: &Rect2D<P>, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::from_rect_in(Global, v, access, alloc)
    }

    #[inline(always)]
    pub fn from_raw (v: &[P::Channel], width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<Self> {
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
    /*#[inline]
    pub fn from_file_in (ctx: C, path: impl AsRef<Path>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynImage {
        let reader = match Reader::open(path) {
            Ok(x) => x,
            Err(e) => return Err(Error::new(ErrorType::InvalidValue, e))
        };

        Self::from_reader_in(ctx, reader, access, alloc)
    }

    /// Creates a new 2D image from an image reader.
    #[inline]
    pub fn from_reader_in<R: BufRead + Seek> (ctx: C, reader: Reader<R>, access: MemAccess, alloc: bool) -> Result<Self> where P: FromDynImage {
        let decode = match reader.decode() {
            Ok(x) => x,
            Err(e) => return Err(Error::new(ErrorType::InvalidValue, e))
        };

        let buffer = P::from_dyn_image(decode);
        Self::from_rect_in(ctx, &buffer, access, alloc)
    }*/

    /// Creates a new 2D image from a 2D rect.
    #[inline(always)]
    pub fn from_rect_in (ctx: C, v: &Rect2D<P>, access: MemAccess, alloc: bool) -> Result<Self> {
        let host = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, v.width(), v.height(), host, NonNull::new(v.as_ptr() as *mut _)) }
    }
    
    /// Creates a new 2D image from it's raw pixels.
    #[inline(always)]
    pub fn from_raw_in (ctx: C, v: &[P::Channel], width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<Self> {
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
        let inner = RawImage::new_2d(ctx.as_raw(), flags, P::FORMAT, desc, host_ptr)?;

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

impl<P: RawPixel + Unpin, C: Context> Image2D<P, C> {
    #[inline(always)]
    pub fn read (&self, slice: impl IntoSlice2D, wait: impl Into<WaitList>) -> Result<ReadImage2D<P>> {
        self.read_with_pitch(slice, None, None, wait)
    }

    #[inline(always)]
    pub fn read_with_pitch (&self, slice: impl IntoSlice2D, row_pitch: Option<usize>, slice_pitch: Option<usize>, wait: impl Into<WaitList>) -> Result<ReadImage2D<P>> {
        unsafe { ReadImage2D::new(self, self.context().next_queue(), slice, row_pitch, slice_pitch, wait) }
    }

    #[inline(always)]
    pub fn write<'src, 'dst> (&'dst mut self, src: &'src Rect2D<P>, offset: [usize; 2], wait: impl Into<WaitList>) -> Result<WriteImage2D<'src, 'dst, P>> {
        self.write_with_pitch(src, offset, None, None, wait)
    }

    #[inline(always)]
    pub fn write_with_pitch<'src, 'dst> (&'dst mut self, src: &'src Rect2D<P>, offset: [usize; 2], row_pitch: Option<usize>, slice_pitch: Option<usize>, wait: impl Into<WaitList>) -> Result<WriteImage2D<'src, 'dst, P>> {
        unsafe { WriteImage2D::new(src, &mut self.inner, self.ctx.next_queue(), offset, row_pitch, slice_pitch, wait) }
    }

    #[inline(always)]
    pub fn copy_from<'src, 'dst> (&'dst mut self, offset_dst: [usize; 2], src: &'src Self, offset_src: [usize; 2], region: [usize; 2], wait: impl Into<WaitList>) -> Result<CopyImage<'src, 'dst>> {
        let mut new_offset_dst = [0; 3];
        let mut new_offset_src = [0; 3];
        let mut new_region = [1; 3];

        unsafe {
            core::ptr::copy_nonoverlapping(offset_dst.as_ptr(), new_offset_dst.as_mut_ptr(), 2);
            core::ptr::copy_nonoverlapping(offset_src.as_ptr(), new_offset_src.as_mut_ptr(), 2);
            core::ptr::copy_nonoverlapping(region.as_ptr(), new_region.as_mut_ptr(), 2);
        }

        unsafe { CopyImage::new(src, new_offset_src, &mut self.inner, new_offset_dst, new_region, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub fn copy_to<'src, 'dst> (&'src self, offset_src: [usize; 2], dst: &'dst mut Self, offset_dst: [usize; 2], region: [usize; 2], wait: impl Into<WaitList>) -> Result<CopyImage<'src, 'dst>> {
        dst.copy_from(offset_dst, self, offset_src, region, wait)
    }

    /*#[inline(always)]
    pub fn fill<'dst> (&'dst mut self, color: impl Borrow<P>, slice: impl IntoSlice2D, wait: impl Into<WaitList>) -> Result<FillImage<'dst>> where f32: FromPrimitive<P::Subpixel> {
        unsafe { FillImage::new(&mut self.inner, color.borrow(), slice, self.ctx.next_queue(), wait) }
    }*/

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

use sealed::Sealed;

mod sealed {
    pub trait Sealed {}
}

pub trait DynImage2D: Sealed {
    fn id_ref (&self) -> &cl_mem; 
}

impl<P: RawPixel, C: Context> DynImage2D for Image2D<P, C> {
    #[inline(always)]
    fn id_ref (&self) -> &cl_mem {
        RawMemObject::id_ref(self)
    }
}

impl<P: RawPixel, C: Context> Sealed for Image2D<P, C> {}