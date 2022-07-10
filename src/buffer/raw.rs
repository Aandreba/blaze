use std::{ptr::{NonNull, addr_of_mut}, ops::{RangeBounds, Bound, Deref}};
use opencl_sys::{cl_mem, clCreateBuffer, CL_FALSE, clEnqueueReadBuffer, clEnqueueWriteBuffer, clEnqueueCopyBuffer};
use rscl_proc::docfg;
use crate::{core::*, context::RawContext, event::{WaitList, RawEvent}};
use super::{flags::{FullMemFlags}};

/// A raw OpenCL memory object
#[repr(transparent)]
pub struct RawBuffer (MemObject);

impl RawBuffer {
    #[inline]
    pub fn new<T> (size: usize, flags: FullMemFlags, host_ptr: Option<NonNull<T>>, ctx: &RawContext) -> Result<Self> {
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

        Ok(Self::from_id(id).unwrap())
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (id: cl_mem) -> Self {
        Self(MemObject::from_id_unchecked(id))
    }

    #[inline(always)]
    pub fn from_id (id: cl_mem) -> Option<Self> {
        let memobj = MemObject::from_id(id)?;
        if memobj.ty() == Ok(MemObjectType::Buffer) {
            return Some(Self(memobj))
        }

        None
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

        Ok(RawBuffer::from_id(id).unwrap())
    }
}

impl RawBuffer {
    /// Reads the contents of this 
    pub unsafe fn read_to_ptr<T: Copy> (&self, src_range: impl RangeBounds<usize>, dst: *mut T, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let (offset, cb) = offset_cb(self, core::mem::size_of::<T>(), src_range)?;
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
    
        let mut event = core::ptr::null_mut();
        tri!(clEnqueueReadBuffer(queue.id(), self.id(), CL_FALSE, offset, cb, dst.cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));
    
        return Ok(RawEvent::from_id(event).unwrap())
    }
    
    pub unsafe fn write_from_ptr<T: Copy> (&mut self, dst_range: impl RangeBounds<usize>, src: *const T, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let (offset, cb) = offset_cb(self, core::mem::size_of::<T>(), dst_range)?;
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
    
        let mut event = core::ptr::null_mut();
        tri!(clEnqueueWriteBuffer(queue.id(), self.id(), CL_FALSE, offset, cb, src.cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));
    
        return Ok(RawEvent::from_id(event).unwrap())
    }

    pub unsafe fn copy_from (&mut self, dst_offset: usize, src: &RawBuffer, src_offset: usize, size: usize, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
    
        let mut event = core::ptr::null_mut();
        tri!(clEnqueueCopyBuffer(queue.id(), src.id(), self.id(), src_offset, dst_offset, size, num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));
    
        return Ok(RawEvent::from_id(event).unwrap())
    }
}

impl Deref for RawBuffer {
    type Target = MemObject;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
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

#[inline]
pub(crate) fn offset_cb (buffer: &RawBuffer, size: usize, range: impl RangeBounds<usize>) -> Result<(usize, usize)> {
    let start = match range.start_bound() {
        Bound::Excluded(x) => x.checked_add(1).and_then(|x| x.checked_mul(size)).unwrap(),
        Bound::Included(x) => x.checked_mul(size).unwrap(),
        Bound::Unbounded => 0
    };

    let end = match range.end_bound() {
        Bound::Excluded(x) => x.checked_mul(size).unwrap(),
        Bound::Included(x) => x.checked_add(1).and_then(|x| x.checked_mul(size)).unwrap(),
        Bound::Unbounded => buffer.size()?
    };

    let len = end - start;
    Ok((start, len))
}

#[inline]
pub(crate) fn range_len (len: usize, range: &impl RangeBounds<usize>) -> usize {
    let start = match range.start_bound() {
        Bound::Excluded(x) => *x + 1,
        Bound::Included(x) => *x,
        Bound::Unbounded => 0
    };

    let end = match range.end_bound() {
        Bound::Excluded(x) => *x,
        Bound::Included(x) => x + 1,
        Bound::Unbounded => len
    };

    end - start
}