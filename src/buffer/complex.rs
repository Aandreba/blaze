use std::{marker::PhantomData, ptr::{NonNull}, ops::{Deref, DerefMut}, fmt::Debug, mem::MaybeUninit, sync::Arc, rc::Rc};
use blaze_proc::docfg;

use crate::{context::{Context, Global}, event::{WaitList}, prelude::{Event, EventExt}};
use crate::core::*;
use crate::buffer::{flags::{MemFlags, HostPtr, MemAccess}, events::{ReadBuffer, WriteBuffer, ReadBufferInto}, RawBuffer};
use super::{events::{CopyBuffer}, IntoRange};

#[derive(Hash)]
#[doc = include_str!("../../docs/src/buffer/README.md")]
pub struct Buffer<T: Copy, C: Context = Global> {
    pub(super) inner: RawBuffer,
    pub(super) ctx: C,
    phtm: PhantomData<T>
}

impl<T: Copy> Buffer<T> {
    /// Creates a new buffer with the given values and flags.
    #[inline(always)]
    pub fn new (v: &[T], access: MemAccess, alloc: bool) -> Result<Self> {
        Self::new_in(Global, v, access, alloc)
    }

    /// Creates a new uninitialized buffer with the given size and flags. 
    #[inline(always)]
    pub fn new_uninit (len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>>> {
        Self::new_uninit_in(Global, len, access, alloc)
    }

    /// Creates a new zero-filled, uninitialized buffer with the given size and flags.
    /// If using OpenCL 1.2 or higher, this uses the `fill` event. Otherwise, a regular `write` is used. 
    #[inline(always)]
    pub fn new_zeroed (len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>>> where T: Unpin {
        Self::new_zeroed_in(Global, len, access, alloc)
    }

    /// Creates a new buffer with the given custom parameters.
    #[inline(always)]
    pub unsafe fn create (len: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(Global, len, flags, host_ptr)
    }
}

impl<T: Copy, C: Context> Buffer<T, C> {
    /// Creates a new buffer with the given values and flags.
    #[inline]
    pub fn new_in (ctx: C, v: &[T], access: MemAccess, alloc: bool) -> Result<Self> {
        let flags = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, v.len(), flags, NonNull::new(v.as_ptr() as *mut _)) }
    }

    /// Creates a new uninitialized buffer with the given size and flags. 
    #[inline(always)]
    pub fn new_uninit_in (ctx: C, len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>, C>> {
        let host = MemFlags::new(access, HostPtr::new(alloc, false));
        unsafe { Buffer::create_in(ctx, len, host, None) }
    }

    /// Creates a new zero-filled, uninitialized buffer with the given size and flags.
    /// If using OpenCL 1.2 or higher, this uses the `fill` event. Otherwise, a regular `write` is used.
    #[inline(always)]
    pub fn new_zeroed_in (ctx: C, len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>, C>> where T: Unpin {
        let mut buffer = Self::new_uninit_in(ctx, len, access, alloc)?;
        #[cfg(feature = "cl1_2")]
        buffer.fill(MaybeUninit::zeroed(), .., WaitList::EMPTY)?.wait()?;
        #[cfg(not(feature = "cl1_2"))]
        buffer.write(0, vec![MaybeUninit::zeroed(); len], WaitList::EMPTY)?.wait()?;
        
        Ok(buffer)
    }

    /// Creates a new buffer with the given custom parameters.
    #[inline]
    pub unsafe fn create_in (ctx: C, len: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();
        let inner = RawBuffer::new_in(&ctx, size, flags, host_ptr.map(NonNull::cast))?;

        Ok(Self {
            inner,
            ctx,
            phtm: PhantomData
        })
    }

    /// Returns the number of elements inside the buffer.
    #[inline(always)]
    pub fn len (&self) -> Result<usize> {
        Ok(self.inner.size()? / core::mem::size_of::<T>())
    }

    /// Reinterprets the bits of the buffer to another type.
    /// # Safety
    /// This function has the same safety as [`transmute`](std::mem::transmute)
    #[inline(always)]
    pub unsafe fn transmute<U: Copy> (self) -> Buffer<U, C> {
        debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());
        Buffer { inner: self.inner, ctx: self.ctx, phtm: PhantomData }
    }

    /// Returns a reference to the buffer's context.
    #[inline(always)]
    pub fn context (&self) -> &C {
        &self.ctx
    }

    /// Checks if the buffer pointer is the same in both buffers.
    #[inline(always)]
    pub fn eq_buffer (&self, other: &Buffer<T, C>) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T: Copy, C: Context> Buffer<MaybeUninit<T>, C> {
    /// Extracts the value from `Buffer<MaybeUninit<T>>` to `Buffer<T>`
    /// # Safety
    /// This function has the same safety as [`MaybeUninit`](std::mem::MaybeUninit)'s `assume_init`
    #[inline(always)]
    pub unsafe fn assume_init (self) -> Buffer<T, C> {
        self.transmute()
    }

    /// Fills the buffer with the given value. Helper function for [`fill`](Self::write)
    #[inline(always)]
    pub fn write_init<'src, 'dst> (&'dst mut self, offset: usize, src: &'src [T], wait: impl Into<WaitList>) -> Result<WriteBuffer<&'src [MaybeUninit<T>], &'dst mut Self>> where T: Unpin {
        debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<MaybeUninit<T>>());
        let src = unsafe { core::slice::from_raw_parts(src.as_ptr().cast(), src.len()) };
        Self::write_by_deref(self, offset, src, wait)
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill_init<'dst> (&'dst mut self, v: T, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::FillBuffer<&'dst mut Self>> where T: Unpin {
        self.fill(MaybeUninit::new(v), range, wait)
    }
}

impl<T: Copy + Unpin, C: Context> Buffer<T, C> {
    /// Returns an event that reads the contents of the buffer into a `Vec<T>`
    #[inline(always)]
    pub fn read<'src> (&'src self, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<ReadBuffer<T, &'src Self>> {
        Self::read_by_deref(self, range, wait)
    }

    /// Returns an event that reads the contents of the buffer into `dst`
    #[inline(always)]
    pub fn read_into<'src, Dst: DerefMut<Target = [T]>> (&'src self, offset: usize, dst: Dst, wait: impl Into<WaitList>) -> Result<ReadBufferInto<&'src Self, Dst>> {
        Self::read_into_by_deref(self, offset, dst, wait)
    }

    /// Returns an event that writes the contents of `src` into the buffer.
    #[inline(always)]
    pub fn write<'dst, Src: Deref<Target = [T]>> (&'dst mut self, offset: usize, src: Src, wait: impl Into<WaitList>) -> Result<WriteBuffer<Src, &'dst mut Self>> {
        Self::write_by_deref(self, offset, src, wait)
    }

    /// Copies the contens from `src` ino the buffer.
    #[inline(always)]
    pub fn copy_from<'dst, Src: Deref<Target = Self>> (&'dst mut self, offset_dst: usize, src: Src, offset_src: usize, len: usize, wait: impl Into<WaitList>) -> Result<CopyBuffer<Src, &'dst mut Self>> {
        Self::copy_from_by_deref(self, offset_dst, src, offset_src, len, wait)
    }

    /// Copies the contents of the buffer into `dst`
    #[inline(always)]
    pub fn copy_to<'src, Dst: DerefMut<Target = Self>> (&'src self, offset_src: usize, dst: Dst, offset_dst: usize, len: usize, wait: impl Into<WaitList>) -> Result<CopyBuffer<&'src Self, Dst>> {
        Self::copy_to_by_deref(self, offset_src, dst, offset_dst, len, wait)
    }

    /// Fills a region of the buffer with `v`
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill<'dst> (&'dst mut self, v: T, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::FillBuffer<&'dst mut Self>> {
        Self::fill_by_deref(self, v, range, wait)
    }

    /// Maps a region of the buffer's device memory into host memory. The mapped region will be read-only.
    #[inline(always)]
    pub fn map<'a> (&'a self, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapBuffer<T, &'a Self>> where T: 'static, C: 'static + Clone {
        Self::map_by_deref(self, range, wait)
    }

    /// Maps a region of the buffer's device memory into host memory. The mapped region will be read-write.
    #[inline(always)]
    pub fn map_mut<'a> (&'a mut self, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapBufferMut<T, &'a mut Self>> where T: 'static, C: 'static {
        Self::map_by_deref_mut(self, range, wait)
    }

    /* RC */

    #[inline(always)]
    pub fn read_local (self: Rc<Self>, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<ReadBuffer<T, Rc<Self>>> {
        Self::read_by_deref(self, range, wait)
    }

    #[inline(always)]
    pub fn read_into_local<Dst: DerefMut<Target = [T]>> (self: Rc<Self>, offset: usize, dst: Dst, wait: impl Into<WaitList>) -> Result<ReadBufferInto<Rc<Self>, Dst>> {
        Self::read_into_by_deref(self, offset, dst, wait)
    }
    
    #[inline(always)]
    pub fn copy_to_local<Dst: DerefMut<Target = Self>> (self: Rc<Self>, offset_src: usize, dst: Dst, offset_dst: usize, len: usize, wait: impl Into<WaitList>) -> Result<CopyBuffer<Rc<Self>, Dst>> {
        Self::copy_to_by_deref(self, offset_src, dst, offset_dst, len, wait)
    }

    /* ARC */

    #[inline(always)]
    pub fn read_owned (self: Arc<Self>, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<ReadBuffer<T, Arc<Self>>> {
        Self::read_by_deref(self, range, wait)
    }

    #[inline(always)]
    pub fn read_into_owned<Dst: DerefMut<Target = [T]>> (self: Arc<Self>, offset: usize, dst: Dst, wait: impl Into<WaitList>) -> Result<ReadBufferInto<Arc<Self>, Dst>> {
        Self::read_into_by_deref(self, offset, dst, wait)
    }
    
    #[inline(always)]
    pub fn copy_to_owned<Dst: DerefMut<Target = Self>> (self: Arc<Self>, offset_src: usize, dst: Dst, offset_dst: usize, len: usize, wait: impl Into<WaitList>) -> Result<CopyBuffer<Arc<Self>, Dst>> {
        Self::copy_to_by_deref(self, offset_src, dst, offset_dst, len, wait)
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

    #[inline(always)]
    pub fn map_by_deref<D: Deref<Target = Self>> (this: D, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapBuffer<T,D>> where T: 'static, C: 'static {
        super::events::MapBuffer::new(this, range, wait)
    }

    #[inline(always)]
    pub fn map_by_deref_mut<D: DerefMut<Target = Self>> (this: D, range: impl IntoRange, wait: impl Into<WaitList>) -> Result<super::events::MapBufferMut<T,D>> where T: 'static, C: 'static {
        super::events::MapBufferMut::new(this, range, wait)
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

impl<T: Copy + Unpin + PartialEq, C: Context> PartialEq for Buffer<T, C> {
    fn eq(&self, other: &Self) -> bool {
        let this = match self.read(.., WaitList::EMPTY) {
            Ok(x) => x,
            Err(_) => return false
        };

        let other = match other.read(.., WaitList::EMPTY) {
            Ok(x) => x,
            Err(_) => return false
        };
        
        let join = match ReadBuffer::join_blocking([this, other]) {
            Ok(x) => x,
            Err(_) => return false
        };

        join[0] == join[1]
    }
}

impl<T: Copy + Unpin + Debug, C: Context> Debug for Buffer<T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.read(.., WaitList::EMPTY).unwrap().wait().unwrap();
        Debug::fmt(&v, f)
    }
}

impl<T: Copy + Unpin + Eq, C: Context> Eq for Buffer<T, C> {}