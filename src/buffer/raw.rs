use std::{mem::MaybeUninit, ptr::{NonNull, addr_of_mut}, ffi::c_void, ops::{RangeBounds, Bound}};
use opencl_sys::{cl_mem, clRetainMemObject, clReleaseMemObject, clGetMemObjectInfo, CL_MEM_OFFSET, CL_MEM_CONTEXT, CL_MEM_REFERENCE_COUNT, CL_MEM_MAP_COUNT, CL_MEM_HOST_PTR, CL_MEM_SIZE, cl_mem_info, clCreateBuffer, CL_MEM_FLAGS, CL_FALSE, clEnqueueReadBuffer, clEnqueueWriteBuffer};
use crate::{core::*, context::RawContext, event::{WaitList, RawEvent}};
use super::{flags::FullMemFlags};

#[repr(transparent)]
pub struct RawBuffer (cl_mem);

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

        Ok(Self::from_id(id))
    }

    #[inline(always)]
    pub const fn from_id (id: cl_mem) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_mem {
        self.0
    }

    #[inline(always)]
    pub const fn id_ref (&self) -> &cl_mem {
        &self.0
    }

    #[inline(always)]
    pub fn flags (&self) -> Result<FullMemFlags> {
        let flags = self.get_info(CL_MEM_FLAGS)?;
        Ok(FullMemFlags::from_bits(flags))
    }

    #[inline(always)]
    pub fn size (&self) -> Result<usize> {
        self.get_info(CL_MEM_SIZE)
    }

    #[inline(always)]
    pub fn host_ptr (&self) -> Result<Option<NonNull<c_void>>> {
        self.get_info(CL_MEM_HOST_PTR).map(NonNull::new)
    }

    /// Map count. The map count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for debugging.
    #[inline(always)]
    pub fn map_count (&self) -> Result<u32> {
        self.get_info(CL_MEM_MAP_COUNT)
    }

    /// Return _memobj_ reference count. The reference count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for identifying memory leaks. 
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_MEM_REFERENCE_COUNT)
    }

    /// Return context specified when memory object is created.
    #[inline(always)]
    pub fn context (&self) -> Result<RawContext> {
        self.get_info(CL_MEM_CONTEXT)
    }

    #[inline(always)]
    pub fn offset (&self) -> Result<usize> {
        self.get_info(CL_MEM_OFFSET)
    }

    #[inline(always)]
    pub unsafe fn clone (&self) -> Self {
        tri_panic!(clRetainMemObject(self.0));
        Self(self.0)
    }

    #[inline]
    pub(super) fn get_info<O> (&self, ty: cl_mem_info) -> Result<O> {
        let mut result = MaybeUninit::<O>::uninit();

        unsafe {
            tri!(clGetMemObjectInfo(self.0, ty, core::mem::size_of::<O>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

impl RawBuffer {
    pub unsafe fn read_to_ptr<T: Copy> (&self, src_range: impl RangeBounds<usize>, dst: *mut T, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let (offset, cb) = offset_cb(self, core::mem::size_of::<T>(), src_range)?;
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
    
        let mut event = core::ptr::null_mut();
        tri!(clEnqueueReadBuffer(queue.id(), self.id(), CL_FALSE, offset, cb, dst.cast(), num_events_in_wait_list, event_wait_list, &mut event));
    
        return Ok(RawEvent::from_id(event))
    }
    
    pub unsafe fn write_from_ptr<T: Copy> (&mut self, dst_range: impl RangeBounds<usize>, src: *const T, queue: &CommandQueue, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let (offset, cb) = offset_cb(self, core::mem::size_of::<T>(), dst_range)?;
        let wait : WaitList = wait.into();
        let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
    
        let mut event = core::ptr::null_mut();
        tri!(clEnqueueWriteBuffer(queue.id(), self.id(), CL_FALSE, offset, cb, src.cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(event)));
    
        return Ok(RawEvent::from_id(event))
    }
}

impl Drop for RawBuffer {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseMemObject(self.0))
        }
    }
}

unsafe impl Send for RawBuffer {}
unsafe impl Sync for RawBuffer {}

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