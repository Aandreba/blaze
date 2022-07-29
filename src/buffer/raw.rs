use std::{ptr::{NonNull, addr_of_mut}, ops::{RangeBounds, Bound, Deref, DerefMut}, ffi::c_void};
use opencl_sys::*;
use blaze_proc::docfg;
use crate::{core::*, context::RawContext, event::{WaitList, RawEvent}, buffer::BufferRange, memobj::{RawMemObject}, prelude::{Global, Context}};
use super::{flags::{MemFlags}, IntoRange};

/// A raw OpenCL buffer
#[repr(transparent)]
#[derive(Clone)]
pub struct RawBuffer (RawMemObject);

impl RawBuffer {
    #[inline(always)]
    pub fn new (size: usize, flags: MemFlags, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        Self::new_in(&Global, size, flags, host_ptr)
    }

    #[inline]
    pub fn new_in (ctx: &RawContext, size: usize, flags: MemFlags, host_ptr: Option<NonNull<c_void>>) -> Result<Self> {
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr().cast(),
            None => core::ptr::null_mut()
        };

        let mut err = 0;
        let id = unsafe {
            clCreateBuffer(ctx.id(), flags.to_bits(), size, host_ptr, addr_of_mut!(err))
        };

        if err != 0 {
            return Err(Error::from(err))
        }

        unsafe { Ok(Self::from_id(id).unwrap()) }
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (id: cl_mem) -> Self {
        Self(RawMemObject::from_id_unchecked(id))
    }

    #[inline(always)]
    pub unsafe fn from_id (id: cl_mem) -> Option<Self> {
        RawMemObject::from_id(id).map(Self)
    }

    /// Creates a new buffer object (referred to as a sub-buffer object) from an existing buffer object.
    #[docfg(feature = "cl1_1")]
    pub fn create_sub_buffer (&self, flags: super::flags::MemAccess, region: impl RangeBounds<usize>) -> Result<RawBuffer> {
        let (origin, size) = offset_cb_plain(self, region)?;
        let region = opencl_sys::cl_buffer_region { origin, size };

        let mut err = 0;
        let id = unsafe {
            opencl_sys::clCreateSubBuffer(self.id(), flags.to_bits(), opencl_sys::CL_BUFFER_CREATE_TYPE_REGION, std::ptr::addr_of!(region).cast(), addr_of_mut!(err))
        };

        if err != 0 {
            return Err(Error::from(err))
        }

        unsafe { Ok(RawBuffer::from_id(id).unwrap()) }
    }
}

impl RawBuffer {
    #[inline(always)]
    pub unsafe fn read_to_ptr<T: Copy> (&self, range: impl IntoRange, dst: *mut T, wait: impl Into<WaitList>) -> Result<RawEvent> {
        self.read_to_ptr_in(range, dst, Global.next_queue(), wait)
    }

    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub unsafe fn read_rect_to_ptr<T: Copy> (
        &self, buffer_origin: [usize; 3], host_origin: [usize;3], region: [usize;3], 
        buffer_row_pitch: Option<usize>, buffer_slice_pitch: Option<usize>, host_row_pitch: Option<usize>,
        host_slice_pitch: Option<usize>, dst: *mut T, wait: impl Into<WaitList>
    ) -> Result<RawEvent> {
        self.read_rect_to_ptr_in(buffer_origin, host_origin, region, buffer_row_pitch, buffer_slice_pitch, host_row_pitch, host_slice_pitch, dst, Global.next_queue(), wait)
    }
    
    #[inline(always)]
    pub unsafe fn write_from_ptr<T: Copy> (&mut self, range: impl IntoRange, src: *const T, wait: impl Into<WaitList>) -> Result<RawEvent> {
        self.write_from_ptr_in(range, src, Global.next_queue(), wait)
    }

    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub unsafe fn write_rect_from_ptr<T: Copy> (
        &mut self, buffer_origin: [usize; 3], host_origin: [usize;3], region: [usize;3], 
        buffer_row_pitch: Option<usize>, buffer_slice_pitch: Option<usize>, host_row_pitch: Option<usize>,
        host_slice_pitch: Option<usize>, src: *const T, wait: impl Into<WaitList>
    ) -> Result<RawEvent> {
        self.write_rect_from_ptr_in(buffer_origin, host_origin, region, buffer_row_pitch, buffer_slice_pitch, host_row_pitch, host_slice_pitch, src, Global.next_queue(), wait)
    }

    #[inline(always)]
    pub unsafe fn copy_from (&mut self, dst_offset: usize, src: &RawBuffer, src_offset: usize, size: usize, wait: impl Into<WaitList>) -> Result<RawEvent> {
        self.copy_from_in(dst_offset, src, src_offset, size, Global.next_queue(), wait)
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub unsafe fn fill_raw<T: Copy> (&mut self, v: T, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<RawEvent> {
        self.fill_raw_in(v, range, Global.next_queue(), wait)
    }

    #[inline(always)]
    pub unsafe fn map_read<T, R: IntoRange, W: Into<WaitList>> (&self, range: R, wait: W) -> Result<(*const T, RawEvent)> {
        self.map_read_in(range, Global.next_queue(), wait)
    }

    #[inline(always)]
    pub unsafe fn map_write<T, R: IntoRange, W: Into<WaitList>> (&self, range: R, wait: W) -> Result<(*mut T, RawEvent)> {
        self.map_write_in(range, Global.next_queue(), wait)
    }

    #[inline(always)]
    pub unsafe fn map_read_write<T, R: IntoRange, W: Into<WaitList>> (&self, range: R, wait: W) -> Result<(*mut T, RawEvent)> {
        self.map_read_write_in(range, Global.next_queue(), wait)
    }
}

impl RawBuffer {
    /// Reads the contents of this 
    pub unsafe fn read_to_ptr_in<T: Copy> (&self, range: impl IntoRange, dst: *mut T, queue: &RawCommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let BufferRange { offset, cb } = range.into_range::<T>(self)?; 
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
    
        let mut event = core::ptr::null_mut();
        tri!(clEnqueueReadBuffer(queue.id(), self.id(), CL_FALSE, offset, cb, dst.cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));
        Ok(RawEvent::from_id(event).unwrap())
    }

    #[docfg(feature = "cl1_1")]
    pub unsafe fn read_rect_to_ptr_in<T: Copy> (
        &self, buffer_origin: [usize; 3], host_origin: [usize;3], region: [usize;3], 
        buffer_row_pitch: Option<usize>, buffer_slice_pitch: Option<usize>, host_row_pitch: Option<usize>,
        host_slice_pitch: Option<usize>, dst: *mut T, queue: &RawCommandQueue, wait: impl Into<WaitList>
    ) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut evt = core::ptr::null_mut();
        tri!(clEnqueueReadBufferRect(queue.id(), self.id(), CL_FALSE, buffer_origin.as_ptr(), host_origin.as_ptr(), region.as_ptr(), buffer_row_pitch.unwrap_or_default(), buffer_slice_pitch.unwrap_or_default(), host_row_pitch.unwrap_or_default(), host_slice_pitch.unwrap_or_default(), dst.cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
        Ok(RawEvent::from_id(evt).unwrap())
    }
    
    pub unsafe fn write_from_ptr_in<T: Copy> (&mut self, range: impl IntoRange, src: *const T, queue: &RawCommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let BufferRange { offset, cb } = range.into_range::<T>(self)?; 
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
    
        let mut event = core::ptr::null_mut();
        tri!(clEnqueueWriteBuffer(queue.id(), self.id(), CL_FALSE, offset, cb, src.cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));
    
        return Ok(RawEvent::from_id(event).unwrap())
    }

    #[docfg(feature = "cl1_1")]
    pub unsafe fn write_rect_from_ptr_in<T: Copy> (
        &mut self, buffer_origin: [usize; 3], host_origin: [usize;3], region: [usize;3], 
        buffer_row_pitch: Option<usize>, buffer_slice_pitch: Option<usize>, host_row_pitch: Option<usize>,
        host_slice_pitch: Option<usize>, src: *const T, queue: &RawCommandQueue, wait: impl Into<WaitList>
    ) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut evt = core::ptr::null_mut();
        tri!(clEnqueueWriteBufferRect(queue.id(), self.id(), CL_FALSE, buffer_origin.as_ptr(), host_origin.as_ptr(), region.as_ptr(), buffer_row_pitch.unwrap_or_default(), buffer_slice_pitch.unwrap_or_default(), host_row_pitch.unwrap_or_default(), host_slice_pitch.unwrap_or_default(), src.cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
        Ok(RawEvent::from_id(evt).unwrap())
    }

    pub unsafe fn copy_from_in (&mut self, dst_offset: usize, src: &RawBuffer, src_offset: usize, size: usize, queue: &RawCommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
    
        let mut event = core::ptr::null_mut();
        tri!(clEnqueueCopyBuffer(queue.id(), src.id(), self.id(), src_offset, dst_offset, size, num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));
    
        return Ok(RawEvent::from_id(event).unwrap())
    }

    #[docfg(feature = "cl1_2")]
    pub unsafe fn fill_raw_in<T: Copy> (&mut self, v: T, range: impl IntoRange, queue: &RawCommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let BufferRange { offset, cb } = range.into_range::<T>(self)?;
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();

        let mut event = core::ptr::null_mut();
        tri!(clEnqueueFillBuffer(queue.id(), self.id(), std::ptr::addr_of!(v).cast(), core::mem::size_of::<T>(), offset, cb, num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));

        Ok(RawEvent::from_id(event).unwrap())
    }

    #[inline(always)]
    pub unsafe fn map_read_in<T, R: IntoRange, W: Into<WaitList>> (&self, range: R, queue: &RawCommandQueue, wait: W) -> Result<(*const T, RawEvent)> {
        let (ptr, evt) = self.__map_inner::<T, R, W, CL_MAP_READ>(range, queue, wait)?;
        Ok((ptr as *const _, evt))
    }

    #[inline(always)]
    pub unsafe fn map_write_in<T, R: IntoRange, W: Into<WaitList>> (&self, range: R, queue: &RawCommandQueue, wait: W) -> Result<(*mut T, RawEvent)> {
        self.__map_inner::<T, R, W, CL_MAP_WRITE>(range, queue, wait)
    }

    #[inline(always)]
    pub unsafe fn map_read_write_in<T, R: IntoRange, W: Into<WaitList>> (&self, range: R, queue: &RawCommandQueue, wait: W) -> Result<(*mut T, RawEvent)> {
        self.__map_inner::<T, R, W, {CL_MAP_READ | CL_MAP_WRITE}>(range, queue, wait)
    }

    unsafe fn __map_inner<T, R: IntoRange, W: Into<WaitList>, const FLAGS : cl_mem_flags> (&self, range: R, queue: &RawCommandQueue, wait: W) -> Result<(*mut T, RawEvent)> {
        let BufferRange { offset, cb } = range.into_range::<T>(self)?;
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
        
        let mut evt = core::ptr::null_mut();
        let mut err = 0;
        let ptr = clEnqueueMapBuffer(queue.id(), self.id(), CL_FALSE, FLAGS, offset, cb, num_events_in_wait_list, event_wait_list, addr_of_mut!(evt), addr_of_mut!(err));
        
        if err != 0 { return Err(Error::from(err)) }
        Ok((ptr.cast(), RawEvent::from_id(evt).unwrap()))
    }
}

impl Deref for RawBuffer {
    type Target = RawMemObject;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawBuffer {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Into<RawMemObject> for RawBuffer {
    #[inline(always)]
    fn into(self) -> RawMemObject {
        self.0
    }
}
 
#[allow(unused)]
#[inline]
pub(crate) fn offset_cb_plain (buffer: &RawBuffer, range: impl RangeBounds<usize>) -> Result<(usize, usize)> {
    let start = match range.start_bound() {
        Bound::Excluded(x) => x.checked_add(1).unwrap(),
        Bound::Included(x) => *x,
        Bound::Unbounded => 0
    };

    let end = match range.end_bound() {
        Bound::Excluded(x) => *x,
        Bound::Included(x) => x.checked_add(1).unwrap(),
        Bound::Unbounded => buffer.size()?
    };

    let len = end - start;
    Ok((start, len))
}