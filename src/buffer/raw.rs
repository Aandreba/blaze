use std::{mem::MaybeUninit, ptr::{NonNull, addr_of_mut}, ffi::c_void, ops::{RangeBounds, Bound, Deref}};
use opencl_sys::{cl_mem, clRetainMemObject, clReleaseMemObject, clGetMemObjectInfo, CL_MEM_CONTEXT, CL_MEM_REFERENCE_COUNT, CL_MEM_MAP_COUNT, CL_MEM_HOST_PTR, CL_MEM_SIZE, cl_mem_info, clCreateBuffer, CL_MEM_FLAGS, CL_FALSE, clEnqueueReadBuffer, clEnqueueWriteBuffer, clCreateSubBuffer, clEnqueueCopyBuffer};
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

        unsafe { Ok(Self::from_id(id).unwrap()) }
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (id: cl_mem) -> Self {
        Self(MemObject::from_id_unchecked(id))
    }

    #[inline(always)]
    pub const fn from_id (id: cl_mem) -> Option<Self> {
        let memobj = MemObject::from_id(id)?;
        if memobj.
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_mem {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub const fn id_ref (&self) -> &cl_mem {
        unsafe { core::mem::transmute(&self.0) }
    }

    /// Return memory object from which memobj is created.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn associated_memobject (&self) -> Result<Option<RawBuffer>> {
        let v = self.get_info::<cl_mem>(opencl_sys::CL_MEM_ASSOCIATED_MEMOBJECT)?;
        Ok(Self::from_id(v))
    }

    /// Return the flags argument value specified when memobj is created.
    #[inline(always)]
    pub fn flags (&self) -> Result<FullMemFlags> {
        let flags = self.get_info(CL_MEM_FLAGS)?;
        Ok(FullMemFlags::from_bits(flags))
    }

    /// Return actual size of the data store associated with memobj in bytes.
    #[inline(always)]
    pub fn size (&self) -> Result<usize> {
        self.get_info(CL_MEM_SIZE)
    }

    /// If memobj is created with a host_ptr specified, return the host_ptr argument value specified when memobj is created.
    #[inline(always)]
    pub fn host_ptr (&self) -> Result<Option<NonNull<c_void>>> {
        self.get_info(CL_MEM_HOST_PTR).map(NonNull::new)
    }

    /// Map count. The map count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for debugging.
    #[inline(always)]
    pub fn map_count (&self) -> Result<u32> {
        self.get_info(CL_MEM_MAP_COUNT)
    }

    /// Return memobj reference count. The reference count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for identifying memory leaks. 
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_MEM_REFERENCE_COUNT)
    }

    /// Return context specified when memory object is created.
    #[inline(always)]
    pub fn context (&self) -> Result<RawContext> {
        self.get_info(CL_MEM_CONTEXT)
    }

    /// Return offset if memobj is a sub-buffer object created using [create_sub_buffer](RawBuffer::create_sub_buffer). Returns 0 if memobj is not a subbuffer object.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn offset (&self) -> Result<usize> {
        self.get_info(opencl_sys::CL_MEM_OFFSET)
    }

    /// Return ```true``` if memobj is a buffer object that was created with CL_MEM_USE_HOST_PTR or is a sub-buffer object of a buffer object that was created with CL_MEM_USE_HOST_PTR and the host_ptr specified when the buffer object was created is a SVM pointer; otherwise returns ```false```.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn uses_svm_pointer (&self) -> Result<bool> {
        let v = self.get_info::<opencl_sys::cl_bool>(opencl_sys::CL_MEM_USES_SVM_POINTER)?;
        Ok(v != 0)
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

    #[inline]
    pub(super) fn get_info<O> (&self, ty: cl_mem_info) -> Result<O> {
        let mut result = MaybeUninit::<O>::uninit();

        unsafe {
            tri!(clGetMemObjectInfo(self.id(), ty, core::mem::size_of::<O>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

#[docfg(feature = "cl1_1")]
impl RawBuffer {
    /// Adds a callback to be executed when the memory object is destructed by OpenCL
    #[inline(always)]
    pub fn on_destruct (&self, f: impl 'static + FnOnce(RawBuffer)) -> Result<()> {
        let f = Box::new(f) as Box<_>;
        self.on_destruct_boxed(f)
    }

    #[inline(always)]
    pub fn on_destruct_boxed (&self, f: Box<dyn FnOnce(RawBuffer)>) -> Result<()> {
        let data = Box::into_raw(Box::new(f));
        unsafe { self.on_destruct_raw(destructor_callback, data.cast()) }
    }

    #[inline(always)]
    pub unsafe fn on_destruct_raw (&self, f: unsafe extern "C" fn(memobj: cl_mem, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        tri!(opencl_sys::clSetMemObjectDestructorCallback(self.id(), Some(f), user_data));
        Ok(())
    }
}

// Buffer methods
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

#[cfg(feature = "cl1_1")]
unsafe extern "C" fn destructor_callback (memobj: cl_mem, user_data: *mut c_void) {
    let f = *Box::from_raw(user_data as *mut Box<dyn FnOnce(RawBuffer)>);
    let memobj = RawBuffer::from_id_unchecked(memobj);
    f(memobj)
}