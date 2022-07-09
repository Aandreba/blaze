use std::{mem::MaybeUninit, ptr::{NonNull}, ffi::c_void, ops::{RangeBounds, Bound}};
use opencl_sys::{cl_mem, clRetainMemObject, clReleaseMemObject, clGetMemObjectInfo, CL_MEM_CONTEXT, CL_MEM_REFERENCE_COUNT, CL_MEM_MAP_COUNT, CL_MEM_HOST_PTR, CL_MEM_SIZE, cl_mem_info, CL_MEM_FLAGS, CL_MEM_OBJECT_BUFFER, CL_MEM_OBJECT_IMAGE2D, CL_MEM_OBJECT_IMAGE3D, CL_MEM_OBJECT_PIPE, CL_MEM_TYPE};
use rscl_proc::docfg;
use crate::{core::*, context::RawContext, buffer::flags::FullMemFlags};

/// A raw OpenCL memory object
#[repr(transparent)]
pub struct MemObject (NonNull<c_void>);

impl MemObject {
    #[inline(always)]
    pub const unsafe fn from_id_unchecked (id: cl_mem) -> Self {
        Self(NonNull::new_unchecked(id))
    }

    #[inline(always)]
    pub const fn from_id (id: cl_mem) -> Option<Self> {
        NonNull::new(id).map(Self)
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_mem {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub const fn id_ref (&self) -> &cl_mem {
        unsafe { core::mem::transmute(&self.0) }
    }

    /// Returns the memory obejct's type
    #[inline(always)]
    pub fn ty (&self) -> Result<MemObjectType> {
        self.get_info(CL_MEM_TYPE)
    }

    /// Return memory object from which memobj is created.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn associated_memobject (&self) -> Result<Option<MemObject>> {
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
impl MemObject {
    /// Adds a callback to be executed when the memory object is destructed by OpenCL
    #[inline(always)]
    pub fn on_destruct (&self, f: impl 'static + FnOnce(MemObject)) -> Result<()> {
        let f = Box::new(f) as Box<_>;
        self.on_destruct_boxed(f)
    }

    #[inline(always)]
    pub fn on_destruct_boxed (&self, f: Box<dyn FnOnce(MemObject)>) -> Result<()> {
        let data = Box::into_raw(Box::new(f));
        unsafe { self.on_destruct_raw(destructor_callback, data.cast()) }
    }

    #[inline(always)]
    pub unsafe fn on_destruct_raw (&self, f: unsafe extern "C" fn(memobj: cl_mem, user_data: *mut c_void), user_data: *mut c_void) -> Result<()> {
        tri!(opencl_sys::clSetMemObjectDestructorCallback(self.id(), Some(f), user_data));
        Ok(())
    }
}

impl Clone for MemObject {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainMemObject(self.id()))
        }

        Self(self.0)
    }
}

impl Drop for MemObject {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseMemObject(self.id()))
        }
    }
}

unsafe impl Send for MemObject {}
unsafe impl Sync for MemObject {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MemObjectType {
    Buffer = CL_MEM_OBJECT_BUFFER,
    Image2D = CL_MEM_OBJECT_IMAGE2D,
    Image3D = CL_MEM_OBJECT_IMAGE3D,
    Pipe = CL_MEM_OBJECT_PIPE
}

impl Into<u32> for MemObjectType {
    #[inline(always)]
    fn into(self) -> u32 {
        self as u32
    }
}

#[allow(unused)]
#[inline]
pub(crate) fn offset_cb_plain (buffer: &MemObject, range: impl RangeBounds<usize>) -> Result<(usize, usize)> {
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

#[cfg(feature = "cl1_1")]
unsafe extern "C" fn destructor_callback (memobj: cl_mem, user_data: *mut c_void) {
    let f = *Box::from_raw(user_data as *mut Box<dyn FnOnce(MemObject)>);
    let memobj = MemObject::from_id_unchecked(memobj);
    f(memobj)
}