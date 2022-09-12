use std::{marker::PhantomData, ptr::{NonNull}, ops::{Deref, DerefMut}, fmt::Debug, mem::MaybeUninit};
use blaze_proc::docfg;
use crate::{context::{Context, Global, Scope, local_scope}, prelude::{Event}, event::consumer::{NoopEvent, Consumer}, WaitList, memobj::MapPtr};
use crate::core::*;
use crate::buffer::{flags::{MemFlags, HostPtr, MemAccess}, RawBuffer};
use super::{IntoRange, BufferRange, MapGuard, BufferMapEvent, BufferMap, BufferMapMutEvent, MapMutGuard, BufferMapMut};

pub type ReadEvent<'a, T> = Event<BufferRead<'a, T>>;

#[doc = include_str!("../../docs/src/buffer/README.md")]
pub struct Buffer<T: Copy, C: Context = Global> {
    pub(super) inner: RawBuffer,
    pub(super) ctx: C,
    pub(super) phtm: PhantomData<T>
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
        buffer.fill_blocking(MaybeUninit::zeroed(), .., WaitList::None)?;
        #[cfg(not(feature = "cl1_2"))]
        buffer.write_blocking(0, &vec![MaybeUninit::zeroed(); len], WaitList::None)?;
        return Ok(buffer)
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

    /// Creates a shared slice of this buffer.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn slice<R: IntoRange> (&self, range: R) -> Result<super::Buf<'_, T, C>> where C: Clone {
        super::Buf::new(self, range)
    }

    /// Creates a mutable slice of this buffer.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn slice_mut<R: IntoRange> (&mut self, range: R) -> Result<super::BufMut<'_, T, C>> where C: Clone {
        super::BufMut::new(self, range)
    }

    /// Reinterprets the bits of the buffer to another type.
    /// # Safety
    /// This function has the same safety as [`transmute`](std::mem::transmute)
    #[inline(always)]
    pub unsafe fn transmute<U: Copy> (self) -> Buffer<U, C> {
        debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());
        Buffer { inner: self.inner, ctx: self.ctx, phtm: PhantomData }
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
}

impl<T: Copy + Unpin, C: Context> Buffer<T, C> {
    /// Reads the contents of the buffer.
    pub fn read<'scope, 'env, R: IntoRange> (&'env self, scope: &'scope Scope<'scope, 'env, C>, range: R, wait: WaitList) -> Result<ReadEvent<'scope, T>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut result = Vec::<T>::with_capacity(len);

        let dst = Vec::as_mut_ptr(&mut result);
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(range, dst.cast(), queue, wait)
        };

        return scope.enqueue(supplier, BufferRead(result, len, PhantomData))
    }

    /// Reads the contents of the buffer, blocking the current thread until the operation has completed.
    pub fn read_blocking<R: IntoRange> (&self, range: R, wait: WaitList) -> Result<Vec<T>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut result = Vec::<T>::with_capacity(len);

        let dst = Vec::as_mut_ptr(&mut result);
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(range, dst.cast(), queue, wait)
        };

        let f = move || unsafe {
            result.set_len(len);
            Ok(result)
        };

        unsafe {
            self.ctx.next_queue().enqueue_unchecked(supplier, f)?.join()
        }
    }

    /// Reads the contents of the buffer into `dst`.
    #[inline]
    pub fn read_into<'scope, 'env, R: IntoRange> (&'env self, s: &'scope Scope<'scope, 'env, C>, offset: usize, dst: &'env mut [T], wait: WaitList) -> Result<NoopEvent<'scope>> {
        let range = BufferRange::from_parts::<T>(offset, dst.len())?;
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(range, dst.as_mut_ptr().cast(), queue, wait)
        };

        return s.enqueue_noop(supplier)
    }

    /// Reads the contents of the buffer into `dst`, blocking the current thread until the operation has completed.
    #[inline]
    pub fn read_into_blocking<R: IntoRange> (&self, offset: usize, dst: &mut [T], wait: WaitList) -> Result<()> {
        let range = BufferRange::from_parts::<T>(offset, dst.len())?;
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(range, dst.as_mut_ptr().cast(), queue, wait)
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop_unchecked(supplier)?.join()
        }
    }

    /// Writes the contents of `src` into the buffer
    #[inline]
    pub fn write<'scope, 'env> (&'scope mut self, scope: &'scope Scope<'scope, 'env, C>, offset: usize, src: &'env [T], wait: WaitList) -> Result<NoopEvent<'scope>> {
        let range = BufferRange::from_parts::<T>(offset, src.len()).unwrap();
        let supplier = |queue| unsafe {
            self.inner.write_from_ptr_in(range, src.as_ptr().cast(), queue, wait)
        };

        scope.enqueue_noop(supplier)
    }

    /// Writes the contents of `src` into the buffer, blocking the current thread until the operation has completed.
    #[inline]
    pub fn write_blocking (&mut self, offset: usize, src: &[T], wait: WaitList) -> Result<()> {
        let range = BufferRange::from_parts::<T>(offset, src.len()).unwrap();
        let supplier = |queue| unsafe {
            self.inner.write_from_ptr_in(range, src.as_ptr().cast(), queue, wait)
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop_unchecked(supplier)?.join()
        }
    }

    /// Copies the contents from `self` to `dst`
    #[inline]
    pub fn copy_to<'scope, 'env> (&'env self, s: &'scope Scope<'scope, 'env, C>, src_offset: usize, dst: &'env mut Self, dst_offset: usize, size: usize, wait: WaitList) -> Result<NoopEvent<'scope>> {
        let src_offset = src_offset.checked_mul(core::mem::size_of::<T>()).unwrap();
        let dst_offset = dst_offset.checked_mul(core::mem::size_of::<T>()).unwrap();
        let size = size.checked_mul(core::mem::size_of::<T>()).unwrap();
        let supplier = |queue| unsafe {
            dst.copy_from_in(dst_offset, &self, src_offset, size, queue, wait)
        };

        s.enqueue_noop(supplier)
    }

    /// Copies the contents from `self` to `dst`, blocking the current thread until the operation has completed.
    #[inline]
    pub fn copy_to_blocking (&self, src_offset: usize, dst: &mut Self, dst_offset: usize, size: usize, wait: WaitList) -> Result<()> {
        let src_offset = src_offset.checked_mul(core::mem::size_of::<T>()).unwrap();
        let dst_offset = dst_offset.checked_mul(core::mem::size_of::<T>()).unwrap();
        let size = size.checked_mul(core::mem::size_of::<T>()).unwrap();
        let supplier = |queue| unsafe {
            dst.copy_from_in(dst_offset, &self, src_offset, size, queue, wait)
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop_unchecked(supplier)?.join()
        }
    }

    /// Copies the contents from `src` to `self`
    #[inline(always)]
    pub fn copy_from<'scope, 'env> (&'env mut self, s: &'scope Scope<'scope, 'env, C>, dst_offset: usize, src: &'env Self, src_offset: usize, size: usize, wait: WaitList) -> Result<NoopEvent<'scope>> {
        src.copy_to(s, src_offset, self, dst_offset, size, wait)
    }

    /// Copies the contents from `src` to `self`, blocking the current thread until the operation has completed.
    #[inline(always)]
    pub fn copy_from_blocking (&mut self, dst_offset: usize, src: &Self, src_offset: usize, size: usize, wait: WaitList) -> Result<()> {
        src.copy_to_blocking(src_offset, self, dst_offset, size, wait)
    }

    /// Fills a region of the buffer with `v`
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill<'scope, 'env, R: IntoRange> (&'env mut self, s: &'scope Scope<'scope, 'env, C>, v: T, range: R, wait: WaitList) -> Result<NoopEvent<'scope>> {
        let range = range.into_range::<T>(&self.inner)?;
        let supplier = |queue| unsafe {
            self.inner.fill_raw_in(v, range, queue, wait)
        };
        
        s.enqueue_noop(supplier)
    }

    /// Fills a region of the buffer with `v`, blocking the current thread until the operation has completed.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill_blocking<R: IntoRange> (&mut self, v: T, range: R, wait: WaitList) -> Result<()> {
        let range = range.into_range::<T>(&self.inner)?;
        let supplier = |queue| unsafe {
            self.inner.fill_raw_in(v, range, queue, wait)
        };
        
        unsafe {
            self.ctx.next_queue().enqueue_noop_unchecked(supplier)?.join()
        }
    }

    pub fn map<'scope, 'env, R: IntoRange> (&'env self, s: &'scope Scope<'scope, 'env, C>, range: R, wait: WaitList) -> Result<BufferMapEvent<'scope, 'env, T, C>> where C: Clone {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();

        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt)
        };

        unsafe {
            let noop = s.enqueue_noop(supplier)?;
            let consumer = BufferMap::new(ptr.assume_init(), self, len);
            return Ok(noop.set_consumer(consumer));
        }
    }

    pub fn map_blocking<'a, R: IntoRange> (&'a self, range: R, wait: WaitList) -> Result<MapGuard<'a, T, C>> where C: Clone {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();
        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt)
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop_unchecked(supplier)?.join()?;
            let ptr = core::slice::from_raw_parts_mut(ptr.assume_init() as *mut T, len);
            let ptr = MapPtr::new(ptr, self.inner.clone().into(), self.ctx.clone());
            return Ok(MapGuard::new(ptr)) 
        }
    }

    pub fn map_mut<'scope, 'env, R: IntoRange> (&'env mut self, s: &'scope Scope<'scope, 'env, C>, range: R, wait: WaitList) -> Result<BufferMapMutEvent<'scope, 'env, T, C>> where C: Clone {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();

        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt)
        };

        unsafe {
            let noop = s.enqueue_noop(supplier)?;
            let consumer = BufferMapMut::new(ptr.assume_init(), self, len);
            return Ok(noop.set_consumer(consumer));
        }
    }

    pub fn map_mut_blocking<'a, R: IntoRange> (&'a mut self, range: R, wait: WaitList) -> Result<MapMutGuard<'a, T, C>> where C: Clone {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();
        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt)
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop_unchecked(supplier)?.join()?;
            let ptr = core::slice::from_raw_parts_mut(ptr.assume_init() as *mut T, len);
            let ptr = MapPtr::new(ptr, self.inner.clone().into(), self.ctx.clone());
            return Ok(MapMutGuard::new(ptr)) 
        }
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
        if self.eq_buffer(other) {
            return true;
        }

        let [this, other] = local_scope(&self.ctx, |s| {
            let this = self.read(s, .., None)?;
            let other = other.read(s, .., None)?;
            Event::join_all_sized_blocking([this, other])
        }).unwrap();

        this == other
    }
}

impl<T: Copy + Unpin + Debug, C: Context> Debug for Buffer<T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.read_blocking(.., None).unwrap();
        Debug::fmt(&v, f)
    }
}

impl<T: Copy + Unpin + Eq, C: Context> Eq for Buffer<T, C> {}

pub struct BufferRead<'a, T> (Vec<T>, usize, PhantomData<&'a RawBuffer>);

impl<'a, T: 'a> Consumer<'a> for BufferRead<'a, T> {
    type Output = Vec<T>;
    
    #[inline(always)]
    fn consume (mut self) -> Result<Vec<T>> {
        unsafe { self.0.set_len(self.1); }
        Ok(self.0)
    }
} 