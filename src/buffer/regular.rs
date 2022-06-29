use std::{marker::PhantomData, ptr::{NonNull}, ops::{RangeBounds, Deref, DerefMut}, ffi::c_void, sync::Arc};
use parking_lot::{FairMutex};
use crate::{context::{Context, Global}, event::{RawEvent, WaitList}};
use crate::core::*;
use super::{flags::{FullMemFlags, HostPtr, MemAccess}, events::{ReadBuffer, WriteBuffer, ReadBufferInto, write_from_static, write_from_ptr}, manager::AccessManager, RawBuffer};

#[cfg(not(debug_assertions))]
use std::hint::unreachable_unchecked;

pub struct Buffer<T: Copy, C: Context = Global> {
    inner: RawBuffer, 
    manager: Arc<FairMutex<AccessManager>>,
    ctx: C,
    phtm: PhantomData<T>
}

impl<T: Copy> Buffer<T> {
    #[inline(always)]
    pub fn new (v: &[T], alloc: bool) -> Result<Self> {
        Self::new_in(v, alloc, Global)
    }

    #[inline(always)]
    pub unsafe fn uninit (len: usize, alloc: bool) -> Result<Self> {
        Self::uninit_in(len, alloc, Global)
    }

    #[inline(always)]
    pub fn create (len: usize, host: HostPtr, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(len, host, host_ptr, Global)
    }
}

impl<T: Copy, C: Context> Buffer<T, C> {
    #[inline]
    pub fn new_in (v: &[T], alloc: bool, ctx: C) -> Result<Self> {
        let host = HostPtr::new(alloc, true);
        Self::create_in(v.len(), host, NonNull::new(v.as_ptr() as *mut _), ctx)
    }

    #[inline(always)]
    pub unsafe fn uninit_in (len: usize, alloc: bool, ctx: C) -> Result<Self> {
        let host = HostPtr::new(alloc, false);
        Self::create_in(len, host, None, ctx)
    }

    #[inline]
    pub fn create_in (len: usize, host: HostPtr, host_ptr: Option<NonNull<T>>, ctx: C) -> Result<Self> {
        let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();
        let inner = RawBuffer::new(size, FullMemFlags::new(MemAccess::READ_WRITE, host), host_ptr, ctx.raw_context())?;

        Ok(Self {
            inner,
            manager: Arc::new(FairMutex::new(AccessManager::None)),
            ctx,
            phtm: PhantomData
        })
    }

    #[inline(always)]
    pub unsafe fn raw (&self) -> &RawBuffer {
        &self.inner
    }

    #[inline(always)]
    pub(crate) fn access_mananer (&self) -> Arc<FairMutex<AccessManager>> {
        self.manager.clone()
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
    pub fn context (&self) -> &C {
        &self.ctx
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
        unsafe { ReadBuffer::new(&self.inner, range, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub fn read_into<P: DerefMut<Target = [T]>> (&self, dst: P, offset: usize, wait: impl Into<WaitList>) -> Result<ReadBufferInto<T, P>> {
        unsafe { ReadBufferInto::new(&self.inner, dst, offset, self.ctx.next_queue(), wait)  }
    }

    #[inline(always)]
    pub fn write<P: Deref<Target = [T]>> (&mut self, src: P, offset: usize, wait: impl Into<WaitList>) -> Result<WriteBuffer<T, P>> {
        unsafe { WriteBuffer::new(src, &mut self.inner, offset, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub fn write_static (&mut self, src: &'static [T], offset: usize, wait: impl Into<WaitList>) -> Result<RawEvent> {
        unsafe { write_from_static(src, &mut self.inner, offset, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub unsafe fn write_ptr (&mut self, src: *const T, range: impl RangeBounds<usize>, wait: impl Into<WaitList>) -> Result<RawEvent> {
        write_from_ptr(src, &mut self.inner, range, self.ctx.next_queue(), wait)
    }
}