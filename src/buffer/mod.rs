flat_mod!(raw);
pub mod flags;
pub mod events;
mod manager;

use std::{marker::PhantomData, ptr::{NonNull, addr_of_mut}, ops::{RangeBounds, Deref, DerefMut}, ffi::c_void};
use opencl_sys::{clCreateBuffer, cl_mem_flags};
use parking_lot::{FairMutex};
use crate::{context::{Context, Global, RawContext}, event::{RawEvent, WaitList}};
use crate::core::*;
use self::{flags::{MemFlags, FullMemFlags, HostPtr}, events::{ReadBuffer, WriteBuffer, ReadBufferInto, write_from_static, write_from_ptr}, manager::AccessManager};

#[cfg(not(debug_assertions))]
use std::hint::unreachable_unchecked;

pub struct Buffer<T: Copy, C: Context = Global> {
    inner: RawBuffer, 
    manager: FairMutex<AccessManager>,
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
        let id = clCreateBuffer(ctx.context().id(), flags.into(), size, host_ptr, addr_of_mut!(err));

        if err != 0 {
            return Err(Error::from(err))
        }

        Ok(Self {
            inner: RawBuffer::from_id(id),
            manager: FairMutex::new(AccessManager::None),
            ctx,
            phtm: PhantomData
        })
    }

    #[inline(always)]
    pub fn len (&self) -> Result<usize> {
        let bytes = self.size()?;
        Ok(bytes / core::mem::size_of::<T>())
    }

    #[inline(always)]
    pub fn size (&self) -> Result<usize> {
        self.inner.size()
    }

    #[inline(always)]
    pub fn host_ptr (&self) -> Result<Option<NonNull<c_void>>> {
        self.inner.host_ptr()
    }

    /// Map count. The map count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for debugging.
    #[inline(always)]
    pub fn map_count (&self) -> Result<u32> {
        self.inner.map_count()
    }

    /// Return _memobj_ reference count. The reference count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for identifying memory leaks. 
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.inner.reference_count()
    }

    /// Return context specified when memory object is created.
    #[inline(always)]
    pub fn context (&self) -> Result<RawContext> {
        self.inner.context()
    }

    #[inline(always)]
    pub fn offset (&self) -> Result<usize> {
        self.inner.offset()
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