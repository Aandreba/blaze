use std::{marker::PhantomData, ptr::{NonNull}, ops::{Deref, DerefMut}, fmt::Debug, mem::MaybeUninit};
use rscl_proc::docfg;

use crate::{context::{Context, Global}, event::{WaitList}, prelude::{Event, EventExt}};
use crate::core::*;
use crate::buffer::{flags::{MemFlags, HostPtr, MemAccess}, events::{ReadBuffer, WriteBuffer, ReadBufferInto}, RawBuffer};

#[cfg(not(debug_assertions))]
use std::hint::unreachable_unchecked;

use super::{events::{CopyBuffer}, IntoRange};

pub struct Buffer<T: Copy, C: Context = Global> {
    pub(super) inner: RawBuffer,
    pub(super) ctx: C,
    phtm: PhantomData<T>
}

impl<T: Copy> Buffer<T> {
    #[inline(always)]
    pub fn new (v: &[T], access: MemAccess, alloc: bool) -> Result<Self> {
        Self::new_in(Global, v, access, alloc)
    }

    #[inline(always)]
    pub fn new_uninit (len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>>> {
        Self::new_uninit_in(Global, len, access, alloc)
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn new_zeroed (len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>>> where T: Unpin {
        Self::new_zeroed_in(Global, len, access, alloc)
    }

    #[inline(always)]
    pub unsafe fn create (len: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(Global, len, flags, host_ptr)
    }
}

impl<T: Copy, C: Context> Buffer<T, C> {
    #[inline]
    pub fn new_in (ctx: C, v: &[T], access: MemAccess, alloc: bool) -> Result<Self> {
        let flags = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, v.len(), flags, NonNull::new(v.as_ptr() as *mut _)) }
    }

    #[inline(always)]
    pub fn new_uninit_in (ctx: C, len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>, C>> {
        let host = MemFlags::new(access, HostPtr::new(alloc, false));
        unsafe { Buffer::create_in(ctx, len, host, None) }
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn new_zeroed_in (ctx: C, len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>, C>> where T: Unpin {
        let mut buffer = Self::new_uninit_in(ctx, len, access, alloc)?;
        buffer.fill(MaybeUninit::zeroed(), .., WaitList::EMPTY)?.wait()?;
        Ok(buffer)
    }

    #[inline]
    pub unsafe fn create_in (ctx: C, len: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();
        let inner = RawBuffer::new_in(&ctx, size, flags, host_ptr)?;

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

impl<T: Copy, C: Context> Buffer<MaybeUninit<T>, C> {
    #[inline(always)]
    pub unsafe fn assume_init (self) -> Buffer<T, C> {
        self.transmute()
    }

    #[inline(always)]
    pub fn write_init<'src, 'dst> (&'dst mut self, offset: usize, src: &'src [T], wait: impl Into<WaitList>) -> Result<WriteBuffer<&'src [MaybeUninit<T>], &'dst mut Self>> where T: Unpin {
        assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<MaybeUninit<T>>());
        let src = unsafe { core::slice::from_raw_parts(src.as_ptr().cast(), src.len()) };
        Self::write_by_deref(self, offset, src, wait)
    }
}

impl<T: Copy + Unpin, C: Context> Buffer<T, C> {
    #[inline(always)]
    pub fn read_all (&self, wait: impl Into<WaitList>) -> Result<ReadBuffer<T, &'_ Self>> {
        self.read(.., wait)
    }

    #[inline(always)]
    pub fn read<'src> (&'src self, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<ReadBuffer<T, &'src Self>> {
        Self::read_by_deref(self, range, wait)
    }

    #[inline(always)]
    pub fn read_into<'src, Dst: DerefMut<Target = [T]>> (&'src self, offset: usize, dst: Dst, wait: impl Into<WaitList>) -> Result<ReadBufferInto<&'src Self, Dst>> {
        Self::read_into_by_deref(self, offset, dst, wait)
    }

    #[inline(always)]
    pub fn write<'dst, Src: Deref<Target = [T]>> (&'dst mut self, offset: usize, src: Src, wait: impl Into<WaitList>) -> Result<WriteBuffer<Src, &'dst mut Self>> {
        Self::write_by_deref(self, offset, src, wait)
    }

    #[inline(always)]
    pub fn copy_from<'dst, Src: Deref<Target = Self>> (&'dst mut self, offset_dst: usize, src: Src, offset_src: usize, len: usize, wait: impl Into<WaitList>) -> Result<CopyBuffer<Src, &'dst mut Self>> {
        Self::copy_from_by_deref(self, offset_dst, src, offset_src, len, wait)
    }

    #[inline(always)]
    pub fn copy_to<'src, Dst: DerefMut<Target = Self>> (&'src self, offset_src: usize, dst: Dst, offset_dst: usize, len: usize, wait: impl Into<WaitList>) -> Result<CopyBuffer<&'src Self, Dst>> {
        Self::copy_to_by_deref(self, offset_src, dst, offset_dst, len, wait)
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill<'dst> (&'dst mut self, v: T, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::FillBuffer<&'dst mut Self>> {
        Self::fill_by_deref(self, v, range, wait)
    }

    #[docfg(feature = "map")]
    #[inline(always)]
    pub fn map<'a> (&'a self, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapBuffer<T, &'a Self, C>> where T: 'static, C: 'static + Clone {
        Self::map_by_deref(self, range, wait)
    }

    #[docfg(feature = "map")]
    #[inline(always)]
    pub fn map_mut<'a> (&'a mut self, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapMutBuffer<T, &'a mut Self, C>> where T: 'static, C: 'static + Clone {
        Self::map_by_deref_mut(self, range, wait)
    }

    /* BY DEREF */

    #[inline(always)]
    pub fn read_by_deref<Src: Deref<Target = Self>> (this: Src, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<ReadBuffer<T, Src>> {
        let queue = this.ctx.next_queue().clone();
        unsafe { ReadBuffer::new(this, range, &queue, wait) }
    }

    #[inline(always)]
    pub fn read_into_by_deref<Src: Deref<Target = Self>, Dst: DerefMut<Target = [T]>> (this: Src, offset: usize, dst: Dst, wait: impl Into<WaitList>) -> Result<ReadBufferInto<Src, Dst>> {
        let queue = this.ctx.next_queue().clone();
        unsafe { ReadBufferInto::new(this, offset, dst, &queue, wait) }
    }

    #[inline(always)]
    pub fn write_by_deref<Dst: DerefMut<Target = Self>, Src: Deref<Target = [T]>> (this: Dst, offset: usize, src: Src, wait: impl Into<WaitList>) -> Result<WriteBuffer<Src, Dst>> {
        let queue = this.ctx.next_queue().clone();
        unsafe { WriteBuffer::new(src, offset, this, &queue, wait) }
    }

    #[inline(always)]
    pub fn copy_from_by_deref<Dst: DerefMut<Target = Self>, Src: Deref<Target = Self>> (this: Dst, offset_dst: usize, src: Src, offset_src: usize, len: usize, wait: impl Into<WaitList>) -> Result<CopyBuffer<Src, Dst>> {
        let queue = this.ctx.next_queue().clone();
        unsafe { CopyBuffer::new(src, offset_src, this, offset_dst, len, &queue, wait) }
    }

    #[inline(always)]
    pub fn copy_to_by_deref<Src: Deref<Target = Self>, Dst: DerefMut<Target = Self>> (this: Src, offset_src: usize, dst: Dst, offset_dst: usize, len: usize, wait: impl Into<WaitList>) -> Result<CopyBuffer<Src, Dst>> {
        Self::copy_from_by_deref(dst, offset_dst, this, offset_src, len, wait)
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill_by_deref<Dst: DerefMut<Target = Self>> (this: Dst, v: T, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::FillBuffer<Dst>> {
        let queue = this.ctx.next_queue().clone();
        unsafe { super::events::FillBuffer::new(v, this, range, &queue, wait) }
    }

    #[docfg(feature = "map")]
    #[inline(always)]
    pub fn map_by_deref<D: Deref<Target = Self>> (this: D, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapBuffer<T,D,C>> where T: 'static, C: 'static + Clone {
        unsafe { super::events::MapBuffer::new(this.ctx.clone(), this, range, wait) }
    }

    #[docfg(feature = "map")]
    #[inline(always)]
    pub fn map_by_deref_mut<D: DerefMut<Target = Self>> (this: D, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapMutBuffer<T,D,C>> where T: 'static, C: 'static + Clone {
        unsafe { super::events::MapMutBuffer::new(this.ctx.clone(), this, range, wait) }
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