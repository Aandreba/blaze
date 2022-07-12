use std::{marker::PhantomData, ptr::{NonNull}, ops::{Deref, DerefMut}, fmt::Debug};
use rscl_proc::docfg;

use crate::{context::{Context, Global}, event::{WaitList}, prelude::Event};
use crate::core::*;
use crate::buffer::{flags::{FullMemFlags, HostPtr, MemAccess}, events::{ReadBuffer, WriteBuffer, ReadBufferInto}, RawBuffer};

#[cfg(not(debug_assertions))]
use std::hint::unreachable_unchecked;

use super::{events::{CopyBuffer}, IntoRange};

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

    #[inline(always)]
    pub unsafe fn transmute<U: Copy> (self) -> Buffer<U, C> {
        assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());
        Buffer { inner: self.inner, ctx: self.ctx, phtm: PhantomData }
    }
}

impl<T: Copy + Unpin, C: Context> Buffer<T, C> {
    #[inline(always)]
    pub fn read_all (&self, wait: impl Into<WaitList>) -> Result<ReadBuffer<T>> {
        self.read(.., wait)
    }

    #[inline(always)]
    pub fn read<'src> (&'src self, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<ReadBuffer<'src, T>> {
        unsafe { ReadBuffer::new(&self.inner, range, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub fn read_into<'src, 'dst> (&'src self, offset: usize, dst: &'dst mut [T], wait: impl Into<WaitList>) -> Result<ReadBufferInto<'src, 'dst>> {
        unsafe { ReadBufferInto::new(&self.inner, offset, dst, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub fn write<'src, 'dst> (&'dst mut self, offset: usize, src: &'src [T], wait: impl Into<WaitList>) -> Result<WriteBuffer<'src, 'dst>> {
        unsafe { WriteBuffer::new(src, offset, &mut self.inner, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub fn copy_from<'src, 'dst, W: Into<WaitList>> (&'dst mut self, offset_dst: usize, src: &'src Self, offset_src: usize, len: usize, wait: W) -> Result<CopyBuffer<'src, 'dst>> {
        unsafe { CopyBuffer::new::<T, W>(&src.inner, offset_src, &mut self.inner, offset_dst, len, self.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub fn copy_to<'src, 'dst, W: Into<WaitList>> (&'src self, offset_src: usize, dst: &'dst mut Self, offset_dst: usize, len: usize, wait: W) -> Result<CopyBuffer<'src, 'dst>> {
        dst.copy_from(offset_dst, self, offset_src, len, wait)
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill<'dst> (&'dst mut self, v: T, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::FillBuffer<'dst>> {
        unsafe { super::events::FillBuffer::new(v, &mut self.inner, range, self.ctx.next_queue(), wait) }
    }

    #[docfg(feature = "map")]
    #[inline(always)]
    pub fn map<'a> (&'a self, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapRefBuffer< 'a,T,C>> where C: Clone {
        unsafe { super::events::MapRefBuffer::new(self.ctx.clone(), self, range, wait) }
    }

    #[docfg(feature = "map")]
    #[inline(always)]
    pub fn map_mut<'a> (&'a mut self, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapMutBuffer< 'a,T,C>> where C: Clone {
        unsafe { super::events::MapMutBuffer::new(self.ctx.clone(), self, range, wait) }
    }
}

impl<T: Copy, C: Context> Deref for Buffer<T, C> {
    type Target = RawBuffer;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Copy, C: Context> DerefMut for Buffer<T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Copy + Unpin + Debug, C: Context> Debug for Buffer<T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.read_all(WaitList::EMPTY).unwrap().wait().unwrap();
        Debug::fmt(&v, f)
    }
}