pub mod flags;
pub mod events;
mod manager;

use std::{marker::PhantomData, ptr::{NonNull, addr_of_mut}, mem::MaybeUninit, ffi::c_void, ops::{RangeBounds, Deref, DerefMut}};
use opencl_sys::{cl_mem, clCreateBuffer, cl_mem_flags, clGetMemObjectInfo, cl_mem_info, CL_MEM_OFFSET, cl_context, CL_MEM_CONTEXT, CL_MEM_REFERENCE_COUNT, CL_MEM_MAP_COUNT, CL_MEM_HOST_PTR, CL_MEM_SIZE, clReleaseMemObject};
use crate::{context::{Context, Global}, event::{RawEvent, WaitList}};
use crate::core::*;
use self::{flags::{MemFlags, FullMemFlags, HostPtr}, events::{ReadBuffer, WriteBuffer, ReadBufferInto, write_from_static, write_from_ptr}};

#[cfg(not(debug_assertions))]
use std::hint::unreachable_unchecked;

pub struct Buffer<T: Copy, C: Context = Global> {
    pub(crate) inner: cl_mem, 
    ctx: C,
    phtm: PhantomData<T>
}

impl<T: Copy> Buffer<T> {
    #[inline(always)]
    pub fn new (v: &[T], flags: impl Into<MemFlags>) -> Result<Self> {
        Self::new_in(v, flags, Global)
    }

    #[inline(always)]
    pub unsafe fn uninit (len: usize, flags: impl Into<MemFlags>) -> Result<Self> {
        Self::uninit_in(len, flags, Global)
    }

    #[inline(always)]
    pub unsafe fn create (len: usize, flags: impl Into<cl_mem_flags>, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(len, flags, host_ptr, Global)
    }
}

impl<T: Copy, C: Context> Buffer<T, C> {
    #[inline]
    pub fn new_in (v: &[T], flags: impl Into<MemFlags>, ctx: C) -> Result<Self> {
        let mut flags : FullMemFlags = flags.into().to_full();
        match flags.host {
            HostPtr::Other(_, ref mut copy) => *copy = true,

            // SAFETY: We converted from ```MemFlags``` to ```FullMemFlags```, which requires that host_ptr be defined by alloc.
            #[cfg(not(debug_assertions))]
            _ => unsafe { unreachable_unchecked() },
            #[cfg(debug_assertions)]
            _ => unreachable!()
        }
        
        unsafe {
            Self::create_in(v.len(), flags, NonNull::new(v.as_ptr() as *mut _), ctx)
        }
    }

    #[inline(always)]
    pub unsafe fn uninit_in (len: usize, flags: impl Into<MemFlags>, ctx: C) -> Result<Self> {
        Self::create_in(len, flags.into(), None, ctx)
    }

    #[inline]
    pub unsafe fn create_in (len: usize, flags: impl Into<cl_mem_flags>, host_ptr: Option<NonNull<T>>, ctx: C) -> Result<Self> {
        let host_ptr = match host_ptr {
            Some(x) => x.as_ptr().cast(),
            None => core::ptr::null_mut()
        };

        let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();
        let mut err = 0;
        let id = clCreateBuffer(ctx.context_id(), flags.into(), size, host_ptr, addr_of_mut!(err));

        if err != 0 {
            return Err(Error::from(err))
        }

        Ok(Self {
            inner: id,
            ctx,
            phtm: PhantomData
        })
    }

    #[inline(always)]
    pub fn len (&self) -> Result<usize> {
        let bytes = self.byte_size()?;
        Ok(bytes / core::mem::size_of::<T>())
    }

    #[inline(always)]
    pub fn byte_size (&self) -> Result<usize> {
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
    pub fn context_id (&self) -> Result<cl_context> {
        let ctx : cl_context = self.get_info(CL_MEM_CONTEXT)?;
        Ok(ctx)
    }

    #[inline(always)]
    pub fn offset (&self) -> Result<usize> {
        self.get_info(CL_MEM_OFFSET)
    }

    #[inline]
    pub(super) fn get_info<O> (&self, ty: cl_mem_info) -> Result<O> {
        let mut result = MaybeUninit::<O>::uninit();

        unsafe {
            tri!(clGetMemObjectInfo(self.inner, ty, core::mem::size_of::<O>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

impl<T: Copy + Unpin, C: Context> Buffer<T, C> {
    #[inline(always)]
    pub fn read_all (&self, wait: impl Into<WaitList>) -> Result<ReadBuffer<T>> {
        self.read(.., wait)
    }

    #[inline(always)]
    pub fn read (&self, range: impl RangeBounds<usize>, wait: impl Into<WaitList>) -> Result<ReadBuffer<T>> {
        ReadBuffer::new(self, range, wait)
    }

    #[inline(always)]
    pub fn read_into<P: DerefMut<Target = [T]>> (&self, dst: P, offset: usize, wait: impl Into<WaitList>) -> Result<ReadBufferInto<T, P>> {
        ReadBufferInto::new(self, dst, offset, wait)
    }

    #[inline(always)]
    pub fn write<P: Deref<Target = [T]>> (&mut self, src: P, offset: usize, wait: impl Into<WaitList>) -> Result<WriteBuffer<T, P>> {
        WriteBuffer::new(src, self, offset, wait)
    }

    // TODO check safety
    #[inline(always)]
    pub fn write_static (&mut self, src: &'static [T], offset: usize, wait: impl Into<WaitList>) -> Result<RawEvent> {
        write_from_static(src, self, offset, wait)
    }

    // TODO check safety
    #[inline(always)]
    pub unsafe fn write_ptr (&mut self, src: *const T, range: impl RangeBounds<usize>, wait: impl Into<WaitList>) -> Result<RawEvent> {
        write_from_ptr(src, self, range, wait)
    }
}

impl<T: Copy, C: Context> Drop for Buffer<T, C> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseMemObject(self.inner))
        }
    }
}

unsafe impl<T: Copy + Send, C: Context + Send> Send for Buffer<T, C> {}
unsafe impl<T: Copy + Sync, C: Context + Sync> Sync for Buffer<T, C> {}