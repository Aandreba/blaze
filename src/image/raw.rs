use opencl_sys::*;
use blaze_proc::docfg;
use std::{ptr::{NonNull, addr_of_mut}, ffi::c_void, ops::{Deref, DerefMut}, mem::MaybeUninit};
use crate::{core::*, context::RawContext, buffer::{flags::MemFlags}, event::WaitList, prelude::RawEvent, memobj::MemObject};
use super::{ImageFormat, ImageDesc};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct RawImage (MemObject);

impl RawImage {
    #[docfg(feature = "cl1_2")]
    pub unsafe fn new (ctx: &RawContext, flags: MemFlags, format: ImageFormat, desc: ImageDesc, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        use std::ptr::addr_of;
        
        let image_format = format.into_raw();
        let image_desc = desc.to_raw();
        
        let flags = flags.to_bits();
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        let id = opencl_sys::clCreateImage(ctx.id(), flags, addr_of!(image_format), addr_of!(image_desc), host_ptr, addr_of_mut!(err));
        
        if err != 0 { return Err(Error::from(err)) }
        let id = MemObject::from_id(id).unwrap();
        Ok(Self(id))
    }

    #[cfg_attr(feature = "cl1_2", deprecated(note = "use `new`"))]
    pub unsafe fn new_2d (ctx: &RawContext, flags: MemFlags, format: ImageFormat, desc: ImageDesc, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        let mut image_format = format.into_raw();
        let flags = flags.to_bits();
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        #[allow(deprecated)]
        let id = opencl_sys::clCreateImage2D(ctx.id(), flags, addr_of_mut!(image_format), desc.width, desc.height, desc.row_pitch, host_ptr, addr_of_mut!(err));
        
        if err != 0 { return Err(Error::from(err)) }
        let id = MemObject::from_id(id).unwrap();
        Ok(Self(id))
    }

    #[cfg_attr(feature = "cl1_2", deprecated(note = "use `new`"))]
    pub unsafe fn new_3d (ctx: &RawContext, flags: MemFlags, format: ImageFormat, desc: ImageDesc, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        let mut image_format = format.into_raw();
        let flags = flags.to_bits();
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        #[allow(deprecated)]
        let id = opencl_sys::clCreateImage3D(ctx.id(), flags, addr_of_mut!(image_format), desc.width, desc.height, desc.depth, desc.row_pitch, desc.slice_pitch, host_ptr, addr_of_mut!(err));
        
        if err != 0 { return Err(Error::from(err)) }
        let id = MemObject::from_id(id).unwrap();
        Ok(Self(id))
    }

    /// Return the image format descriptor specified when image is created.
    #[inline(always)]
    pub fn format (&self) -> Result<ImageFormat> {
        let v = self.get_info::<cl_image_format>(CL_IMAGE_FORMAT)?;
        Ok(ImageFormat::from_raw(v).unwrap())
    }

    /// Return size of each element of the image memory object given by image in bytes.
    #[inline(always)]
    pub fn element_size (&self) -> Result<usize> {
        self.get_info(CL_IMAGE_ELEMENT_SIZE)
    }

    /// Returns the row pitch in bytes of a row of elements of the image object given by image.
    /// If image was created with a non-zero value for image_row_pitch, then the value provided for image_row_pitch by the application is returned, 
    /// otherwise the returned value is calculated as [`RawImage::width`] × [`RawImage::element_size`].
    #[inline(always)]
    pub fn row_pitch (&self) -> Result<usize> {
        self.get_info(CL_IMAGE_ROW_PITCH)
    }

    /// Returns the slice pitch in bytes of a 2D slice for the 3D image object or size of each image in a 1D or 2D image array given by image.
    /// If image was created with a non-zero value for image_slice_pitch then the value provided for image_slice_pitch by the application is returned, otherwise the returned value is calculated as:
    /// - [`RawImage::row_pitch`] for 1D image arrays.
    /// - [`RawImage::height`] × [`RawImage::row_pitch`] for 3D images and 2D image arrays.\
    /// For a 1D image, 1D image buffer and 2D image object return 0.
    #[inline(always)]
    pub fn slice_pitch (&self) -> Result<usize> {
        self.get_info(CL_IMAGE_SLICE_PITCH)
    }

    /// Return width of the image in pixels.
    #[inline(always)]
    pub fn width (&self) -> Result<usize> {
        self.get_info(CL_IMAGE_WIDTH)
    }

    /// Return height of the image in pixels. For a 1D image, 1D image buffer and 1D image array object, height is zero.
    #[inline(always)]
    pub fn height (&self) -> Result<usize> {
        self.get_info(CL_IMAGE_HEIGHT)
    }

    /// Return depth of the image in pixels. For a 1D image, 1D image buffer, 2D image or 1D and 2D image array object, depth is zero.
    #[inline(always)]
    pub fn depth (&self) -> Result<usize> {
        self.get_info(CL_IMAGE_DEPTH)
    }

    /// Return number of images in the image array. If image is not an image array, 0 is returned.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn array_size (&self) -> Result<usize> {
        self.get_info(opencl_sys::CL_IMAGE_ARRAY_SIZE)
    }

    /// Return buffer object associated with image.
    #[docfg(feature = "cl1_2")]
    #[cfg_attr(feature = "2", deprecated(note = "use `other`"))]
    #[inline(always)]
    pub fn buffer (&self) -> Result<Option<MemObject>> {
        self.get_info::<cl_mem>(opencl_sys::CL_IMAGE_ARRAY_SIZE).map(MemObject::from_id)
    }

    /// Return `num_mip_levels` associated with image.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn mip_levels (&self) -> Result<usize> {
        self.get_info(opencl_sys::CL_IMAGE_NUM_MIP_LEVELS)
    }

    /// Return `num_samples` associated with image.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn samples (&self) -> Result<usize> {
        self.get_info(opencl_sys::CL_IMAGE_NUM_SAMPLES)
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_image_info) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();

        unsafe {
            tri!(clGetImageInfo(self.id(), ty, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

impl RawImage {
    #[inline]
    pub unsafe fn read_to_ptr (&self, origin: [usize; 3], region: [usize; 3], row_pitch: Option<usize>, slice_pitch: Option<usize>, dst: *mut c_void, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let row_pitch = row_pitch.unwrap_or_default();
        let slice_pitch = slice_pitch.unwrap_or_default();

        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
        
        let mut evt = core::ptr::null_mut();
        tri!(clEnqueueReadImage(queue.id(), self.id(), CL_FALSE, origin.as_ptr(), region.as_ptr(), row_pitch, slice_pitch, dst, num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
        Ok(RawEvent::from_id(evt).unwrap())
    }

    #[inline]
    pub unsafe fn write_from_ptr (&mut self, origin: [usize; 3], region: [usize; 3], row_pitch: Option<usize>, slice_pitch: Option<usize>, src: *const c_void, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let row_pitch = row_pitch.unwrap_or_default();
        let slice_pitch = slice_pitch.unwrap_or_default();

        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
        
        let mut evt = core::ptr::null_mut();
        tri!(clEnqueueWriteImage(queue.id(), self.id(), CL_FALSE, origin.as_ptr(), region.as_ptr(), row_pitch, slice_pitch, src, num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
        Ok(RawEvent::from_id(evt).unwrap())
    }

    #[inline]
    pub unsafe fn copy_from (&mut self, offset_dst: [usize; 3], src: &RawImage, offset_src: [usize; 3], region: [usize; 3], queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
        
        let mut evt = core::ptr::null_mut();
        tri!(clEnqueueCopyImage(queue.id(), src.id(), self.id(), offset_src.as_ptr(), offset_dst.as_ptr(), region.as_ptr(), num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
        Ok(RawEvent::from_id(evt).unwrap())
    }

    #[inline(always)]
    pub unsafe fn copy_to (&self, offset_src: [usize; 3], dst: &mut RawImage, offset_dst: [usize; 3], region: [usize; 3], queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        Self::copy_from(dst, offset_dst, self, offset_src, region, queue, wait)
    }

    #[docfg(feature = "cl1_2")]
    #[inline]
    pub unsafe fn fill (&mut self, color: *const c_void, origin: [usize; 3], region: [usize; 3], queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
        
        let mut evt = core::ptr::null_mut();
        tri!(clEnqueueFillImage(queue.id(), self.id(), color, origin.as_ptr(), region.as_ptr(), num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
        Ok(RawEvent::from_id(evt).unwrap())
    }

    #[inline(always)]
    pub unsafe fn map_read<T, W: Into<WaitList>> (&self, origin: [usize; 3], region: [usize; 3], queue: &CommandQueue, wait: W) -> Result<(*const T, usize, usize, RawEvent)> {
        let (ptr, image_row_pitch, image_slice_pitch, evt) = self.__map_inner::<T, W, CL_MAP_READ>(origin, region, queue, wait)?;
        Ok((ptr as *const _, image_row_pitch, image_slice_pitch, evt))
    }

    #[inline(always)]
    pub unsafe fn map_write<T, W: Into<WaitList>> (&self, origin: [usize; 3], region: [usize; 3], queue: &CommandQueue, wait: W) -> Result<(*mut T, usize, usize, RawEvent)> {
        self.__map_inner::<T, W, CL_MAP_WRITE>(origin, region, queue, wait)
    }

    #[inline(always)]
    pub unsafe fn map_read_write<T, W: Into<WaitList>> (&self, origin: [usize; 3], region: [usize; 3], queue: &CommandQueue, wait: W) -> Result<(*mut T, usize, usize, RawEvent)> {
        self.__map_inner::<T, W, {CL_MAP_READ | CL_MAP_WRITE}>(origin, region, queue, wait)
    }

    unsafe fn __map_inner<T, W: Into<WaitList>, const FLAGS : cl_mem_flags> (&self, origin: [usize; 3], region: [usize; 3], queue: &CommandQueue, wait: W) -> Result<(*mut T, usize, usize, RawEvent)> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
        
        let mut image_row_pitch = 0;
        let mut image_slice_pitch = 0;
        let mut evt = core::ptr::null_mut();
        let mut err = 0;

        let ptr = clEnqueueMapImage(queue.id(), self.id(), CL_FALSE, FLAGS, origin.as_ptr(), region.as_ptr(), addr_of_mut!(image_row_pitch), addr_of_mut!(image_slice_pitch), num_events_in_wait_list, event_wait_list, addr_of_mut!(evt), addr_of_mut!(err));
        if err != 0 { return Err(Error::from(err)) }
        Ok((ptr.cast(), image_row_pitch, image_slice_pitch, RawEvent::from_id(evt).unwrap()))
    }
}

impl Deref for RawImage {
    type Target = MemObject;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawImage {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Into<MemObject> for RawImage {
    #[inline(always)]
    fn into(self) -> MemObject {
        self.0
    }
}