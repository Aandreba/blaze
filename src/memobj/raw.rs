use super::MemObjectType;
use crate::{buffer::flags::MemFlags, context::RawContext, core::*, non_null_const};
use blaze_proc::docfg;
use opencl_sys::*;
use std::{ffi::c_void, mem::MaybeUninit, ptr::NonNull};

/// A raw OpenCL memory object
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RawMemObject(NonNull<c_void>);

impl RawMemObject {
    #[inline(always)]
    pub const unsafe fn from_id_unchecked(id: cl_mem) -> Self {
        Self(NonNull::new_unchecked(id))
    }

    #[inline(always)]
    pub const unsafe fn from_id(id: cl_mem) -> Option<Self> {
        match non_null_const(id) {
            Some(ptr) => Some(Self(ptr)),
            None => None,
        }
    }

    #[inline(always)]
    pub unsafe fn retain(&self) -> Result<()> {
        tri!(clRetainMemObject(self.id()));
        Ok(())
    }

    #[inline(always)]
    pub const fn id(&self) -> cl_mem {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub const fn id_ref(&self) -> &cl_mem {
        unsafe { core::mem::transmute(&self.0) }
    }

    #[inline(always)]
    pub fn id_ref_mut(&mut self) -> &mut cl_mem {
        unsafe { core::mem::transmute(&mut self.0) }
    }

    /// Returns the memory obejct's type
    #[inline(always)]
    pub fn ty(&self) -> Result<MemObjectType> {
        self.get_info(CL_MEM_TYPE)
    }

    /// Return memory object from which memobj is created.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn associated_memobject(&self) -> Result<Option<RawMemObject>> {
        let v = self.get_info::<cl_mem>(opencl_sys::CL_MEM_ASSOCIATED_MEMOBJECT)?;
        unsafe {
            if let Some(id) = Self::from_id(v) {
                id.retain()?;
                return Ok(Some(id));
            }

            return Ok(None);
        }
    }

    /// Return the flags argument value specified when memobj is created.
    #[inline(always)]
    pub fn flags(&self) -> Result<MemFlags> {
        let flags = self.get_info(CL_MEM_FLAGS)?;
        Ok(MemFlags::from_bits(flags))
    }

    /// Return actual size of the data store associated with memobj in bytes.
    #[inline(always)]
    pub fn size(&self) -> Result<usize> {
        self.get_info(CL_MEM_SIZE)
    }

    /// If memobj is created with a host_ptr specified, return the host_ptr argument value specified when memobj is created.
    #[inline(always)]
    pub fn host_ptr(&self) -> Result<Option<NonNull<c_void>>> {
        self.get_info(CL_MEM_HOST_PTR).map(NonNull::new)
    }

    /// Map count. The map count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for debugging.
    #[inline(always)]
    pub fn map_count(&self) -> Result<u32> {
        self.get_info(CL_MEM_MAP_COUNT)
    }

    /// Return memobj reference count. The reference count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for identifying memory leaks.
    #[inline(always)]
    pub fn reference_count(&self) -> Result<u32> {
        self.get_info(CL_MEM_REFERENCE_COUNT)
    }

    /// Return context specified when memory object is created.
    #[inline(always)]
    pub fn context(&self) -> Result<RawContext> {
        let ctx = self.get_info::<cl_context>(CL_MEM_CONTEXT)?;
        unsafe {
            tri!(clRetainContext(ctx));
            // SAFETY: Context checked to be valid by `clRetainContext`.
            Ok(RawContext::from_id_unchecked(ctx))
        }
    }

    /// Return offset if memobj is a sub-buffer object created using [create_sub_buffer](crate::buffer::RawBuffer::create_sub_buffer). Returns 0 if memobj is not a subbuffer object.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn offset(&self) -> Result<usize> {
        self.get_info(opencl_sys::CL_MEM_OFFSET)
    }

    /// Return ```true``` if memobj is a buffer object that was created with CL_MEM_USE_HOST_PTR or is a sub-buffer object of a buffer object that was created with CL_MEM_USE_HOST_PTR and the host_ptr specified when the buffer object was created is a SVM pointer; otherwise returns ```false```.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn uses_svm_pointer(&self) -> Result<bool> {
        let v = self.get_info::<opencl_sys::cl_bool>(opencl_sys::CL_MEM_USES_SVM_POINTER)?;
        Ok(v != 0)
    }

    #[inline]
    pub(super) fn get_info<O: Copy>(&self, ty: cl_mem_info) -> Result<O> {
        let mut result = MaybeUninit::<O>::uninit();

        unsafe {
            tri!(clGetMemObjectInfo(
                self.id(),
                ty,
                core::mem::size_of::<O>(),
                result.as_mut_ptr().cast(),
                core::ptr::null_mut()
            ));
            Ok(result.assume_init())
        }
    }
}

#[docfg(feature = "cl1_1")]
impl RawMemObject {
    /// Adds a callback to be executed when the memory object is destructed by OpenCL.
    #[inline(always)]
    pub fn on_destruct(&self, f: impl 'static + FnOnce() + Send) -> Result<()> {
        let f = Box::new(f) as Box<_>;
        self.on_destruct_boxed(f)
    }

    #[inline(always)]
    pub fn on_destruct_boxed(&self, f: Box<dyn FnOnce() + Send>) -> Result<()> {
        let data = Box::into_raw(Box::new(f));
        unsafe { self.on_destruct_raw(destructor_callback, data.cast()) }
    }

    #[inline(always)]
    pub unsafe fn on_destruct_raw(
        &self,
        f: unsafe extern "C" fn(memobj: cl_mem, user_data: *mut c_void),
        user_data: *mut c_void,
    ) -> Result<()> {
        tri!(opencl_sys::clSetMemObjectDestructorCallback(
            self.id(),
            Some(f),
            user_data
        ));
        Ok(())
    }
}

impl Clone for RawMemObject {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe { tri_panic!(clRetainMemObject(self.id())) }

        Self(self.0)
    }
}

impl Drop for RawMemObject {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { tri_panic!(clReleaseMemObject(self.id())) }
    }
}

unsafe impl Send for RawMemObject {}
unsafe impl Sync for RawMemObject {}

#[cfg(feature = "cl1_1")]
unsafe extern "C" fn destructor_callback(_memobj: cl_mem, user_data: *mut c_void) {
    let f = *Box::from_raw(user_data as *mut Box<dyn FnOnce() + Send>);
    f()
}
