use std::{mem::MaybeUninit, ptr::{NonNull, addr_of_mut}, ffi::c_void};
use opencl_sys::{cl_mem, clRetainMemObject, clReleaseMemObject, clGetMemObjectInfo, CL_MEM_OFFSET, CL_MEM_CONTEXT, CL_MEM_REFERENCE_COUNT, CL_MEM_MAP_COUNT, CL_MEM_HOST_PTR, CL_MEM_SIZE, cl_mem_info, clCreateBuffer, CL_MEM_FLAGS};
use crate::{core::*, context::RawContext};
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

    #[inline]
    pub(super) fn get_info<O> (&self, ty: cl_mem_info) -> Result<O> {
        let mut result = MaybeUninit::<O>::uninit();

        unsafe {
            tri!(clGetMemObjectInfo(self.0, ty, core::mem::size_of::<O>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

impl Clone for RawBuffer {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainMemObject(self.0))
        }

        Self(self.0)
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