use std::{marker::PhantomData, ptr::{NonNull}, ops::{RangeBounds, Deref, DerefMut}, sync::Arc};

use crate::{context::{Context, Global}, event::{RawEvent, WaitList}, prelude::Event};
use crate::core::*;
use crate::buffer::{flags::{FullMemFlags, HostPtr, MemAccess}, events::{ReadBuffer, WriteBuffer, ReadBufferInto, write_from_static, write_from_ptr}, manager::AccessManager, RawBuffer};

#[cfg(not(debug_assertions))]
use std::hint::unreachable_unchecked;

use super::offset_cb;

pub struct Buffer<T: Copy, C: Context = Global> {
    inner: RawBuffer,
    ctx: C,
    phtm: PhantomData<T>
}

impl<T: Copy> Buffer<T> {
    #[inline(always)]
    pub fn new (v: &[T], access: MemAccess, alloc: bool) -> Result<Self> {
        Self::new_in(Global, v, access, alloc)
    }

    #[inline(always)]
    pub unsafe fn uninit (len: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::uninit_in(Global, len, access, alloc)
    }

    #[inline(always)]
    pub fn create (len: usize, flags: FullMemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(Global, len, flags, host_ptr)
    }
}

impl<T: Copy, C: Context> Buffer<T, C> {
    #[inline]
    pub fn new_in (ctx: C, v: &[T], access: MemAccess, alloc: bool) -> Result<Self> {
        let flags = FullMemFlags::new(access, HostPtr::new(alloc, true));
        Self::create_in(ctx, v.len(), flags, NonNull::new(v.as_ptr() as *mut _))
    }

    #[inline(always)]
    pub unsafe fn uninit_in (ctx: C, len: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        let host = FullMemFlags::new(access, HostPtr::new(alloc, false));
        Self::create_in(ctx, len, host, None)
    }

    #[inline]
    pub fn create_in (ctx: C, len: usize, flags: FullMemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();
        let inner = RawBuffer::new(size, flags, host_ptr, ctx.as_raw())?;

        Ok(Self {
            inner,
            ctx,
            phtm: PhantomData
        })
    }
}

impl<T: Copy + Unpin, C: Context> Buffer<T, C> {
    #[inline(always)]
    fn read_all (&self, wait: impl Into<WaitList>) -> Result<ReadBuffer<T>> {
        self.read(.., wait)
    }

    #[inline(always)]
    fn read<'src> (&'src self, range: impl RangeBounds<usize>, wait: impl Into<WaitList>) -> Result<ReadBuffer<'src, T>> {
        unsafe { ReadBuffer::new(&self.inner, range, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    fn read_into<'src, 'dst> (&'src self, dst: &'dst mut [T], offset: usize, wait: impl Into<WaitList>) -> Result<ReadBufferInto<'src, 'dst>> {
        unsafe { ReadBufferInto::new(&self.inner, dst, offset, self.ctx.next_queue(), wait) }
    }

    #[inline]
    fn write<P: Deref<Target = [T]>> (&mut self, src: P, offset: usize, wait: impl Into<WaitList>) -> Result<WriteBuffer<T, P>> {
        let access = self.access_mananer();
        let mut access = access.lock();

        let mut wait : WaitList = wait.into();
        access.extend_to_write(&mut wait);

        let queue = self.context().next_queue().clone();
        let evt = unsafe { WriteBuffer::new(src, self.as_mut(), offset, &queue, wait)? };
        access.write(evt.to_raw());

        Ok(evt)
    }

    #[inline]
    fn write_static (&mut self, src: &'static [T], offset: usize, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let access = self.access_mananer();
        let mut access = access.lock();

        let mut wait : WaitList = wait.into();
        access.extend_to_write(&mut wait);

        let queue = self.context().next_queue().clone();
        let evt = unsafe { write_from_static(src, self.as_mut(), offset, &queue, wait)? };
        access.write(evt.clone());

        Ok(evt)
    }

    #[inline]
    unsafe fn write_ptr (&mut self, src: *const T, range: impl RangeBounds<usize>, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let access = self.access_mananer();
        let mut access = access.lock();

        let mut wait : WaitList = wait.into();
        access.extend_to_write(&mut wait);

        let queue = self.context().next_queue().clone();
        let evt = write_from_ptr(src, self.as_mut(), range, &queue, wait)?;
        access.write(evt.clone());

        Ok(evt)
    }

    #[inline(always)]
    fn copy_from (&mut self, offset: usize, src: &Self, range: impl RangeBounds<usize>, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let dst_offset = offset.checked_mul(core::mem::size_of::<T>()).unwrap();
        let (src_offset, size) = offset_cb(src.as_ref(), core::mem::size_of::<T>(), range)?;

        let (dst_access, src_access) = (self.access_mananer(), src.access_mananer());
        let mut src_access = src_access.lock();
        let mut dst_access = dst_access.lock();

        let mut wait : WaitList = wait.into();
        src_access.extend_to_read(&mut wait);
        dst_access.extend_to_write(&mut wait);

        let queue = self.context().next_queue().clone();
        let evt = unsafe { self.as_mut().copy_from(dst_offset, src.as_ref(), src_offset, size, &queue, wait)? };
        src_access.read(evt.clone());
        dst_access.write(evt.clone());

        Ok(evt)
    }

    #[inline(always)]
    fn copy_to (&self, range: impl RangeBounds<usize>, dst: &mut Self, offset: usize, wait: impl Into<WaitList>) -> Result<RawEvent> {
        dst.copy_from(offset, self, range, wait)
    }
}