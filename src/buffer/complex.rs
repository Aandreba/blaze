use std::{marker::PhantomData, ptr::{NonNull}, ops::{Deref, DerefMut}, fmt::Debug, mem::{MaybeUninit, transmute}};
use blaze_proc::docfg;
use crate::{context::{Context, Global, Scope, local_scope}, prelude::{Event}, event::consumer::{Consumer, PhantomEvent}, WaitList, memobj::MapPtr};
use crate::core::*;
use crate::buffer::{flags::{MemFlags, HostPtr, MemAccess}, RawBuffer};
use super::{IntoRange, BufferRange, MapGuard, BufferMapEvent, BufferMap, BufferMapMutEvent, MapMutGuard, BufferMapMut};

pub type ReadEvent<'a, T, C = Global> = Event<BufferRead<'a, T, C>>;
pub type ReadIntoEvent<'a, T, C = Global> = PhantomEvent<(&'a Buffer<T, C>, &'a mut [T])>;
pub type WriteEvent<'a, T, C = Global> = PhantomEvent<(&'a mut Buffer<T, C>, &'a [T])>;
pub type CopyEvent<'a, T, C = Global> = PhantomEvent<(&'a Buffer<T, C>, &'a mut Buffer<T, C>)>;
pub type FillEvent<'a, T, C = Global> = PhantomEvent<(&'a mut Buffer<T, C>, T)>;

#[doc = include_str!("../../docs/src/buffer/README.md")]
pub struct Buffer<T, C: Context = Global> {
    pub(super) inner: RawBuffer,
    pub(super) ctx: C,
    pub(super) phtm: PhantomData<T>
}

impl<T> Buffer<T> {
    /// Creates a new buffer with the given values and flags.
    #[inline(always)]
    pub fn new (v: &[T], access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
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
    pub fn new_zeroed (len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>>> {
        Self::new_zeroed_in(Global, len, access, alloc)
    }

    /// Creates a new buffer with the given custom parameters.
    #[inline(always)]
    pub unsafe fn create (len: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(Global, len, flags, host_ptr)
    }
}

impl<T, C: Context> Buffer<T, C> {
    /// Creates a new buffer with the given values and flags.
    #[inline]
    pub fn new_in (ctx: C, v: &[T], access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
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
    pub fn new_zeroed_in (ctx: C, len: usize, access: MemAccess, alloc: bool) -> Result<Buffer<MaybeUninit<T>, C>> {
        let mut buffer = Self::new_uninit_in(ctx, len, access, alloc)?;
        #[cfg(feature = "cl1_2")]
        {
            let range = (..).into_range::<T>(&buffer)?;
            let supplier = |queue| unsafe {
                buffer.inner.fill_raw_in(MaybeUninit::<T>::zeroed(), range, queue, None)
            };
            buffer.ctx.next_queue().enqueue_noop(supplier)?.join()?;
        }
        #[cfg(not(feature = "cl1_2"))]
        {
            let mut v = Vec::<T>::with_capacity(len);
            unsafe {
                core::ptr::write_bytes(v.as_mut_ptr(), 0, len);
            }

            let range = BufferRange::from_parts::<T>(0, len).unwrap();
            let supplier = |queue| unsafe {
                buffer.inner.write_from_ptr_in(range, v.as_ptr().cast(), queue, None)
            };

            buffer.ctx.next_queue().enqueue_noop(supplier)?.join()?;
        }
        return Ok(buffer)
    }

    /// Creates a new buffer with the given custom parameters.
    #[inline]
    pub unsafe fn create_in (ctx: C, len: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();
        let inner = RawBuffer::new_in(ctx.as_raw(), size, flags, host_ptr.map(NonNull::cast))?;

        Ok(Self {
            inner,
            ctx,
            phtm: PhantomData
        })
    }

    /// Number of elements inside the buffer
    #[inline(always)]
    pub fn len (&self) -> Result<usize> {
        self.size().map(|x| x / core::mem::size_of::<T>())
    }

    /// Returns a reference to the [`Buffer`]'s underlying [`Context`].
    #[inline(always)]
    pub fn context (&self) -> &C {
        &self.ctx
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
    pub unsafe fn transmute<U> (self) -> Buffer<U, C> {
        debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());
        Buffer { inner: self.inner, ctx: self.ctx, phtm: PhantomData }
    }

    /// Checks if the buffer pointer is the same in both buffers.
    #[inline(always)]
    pub fn eq_buffer (&self, other: &Buffer<T, C>) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T, C: Context> Buffer<MaybeUninit<T>, C> {
    /// Convenience method for writing to an unitialized buffer. See [`write`](Buffer::write).
    #[inline(always)]
    pub fn write_init<'scope, 'env> (&'env mut self, scope: &'scope Scope<'scope, 'env, C>, offset: usize, src: &'env [T], wait: WaitList) -> Result<WriteEvent<'scope, MaybeUninit<T>, C>> where T: Copy {
        let src = unsafe { transmute::<&'env [T], &'env [MaybeUninit<T>]>(src) };
        self.write(scope, offset, src, wait)
    }

    /// Convenience method for filling an unitialized buffer. See [`fill`](Buffer::fill).
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill_init<'scope, 'env, R: IntoRange> (&'env mut self, scope: &'scope Scope<'scope, 'env, C>, v: T, range: R, wait: WaitList) -> Result<FillEvent<'scope, MaybeUninit<T>, C>> where T: Copy {
        self.fill(scope, MaybeUninit::new(v), range, wait)
    }

    /// Extracts the value from `Buffer<MaybeUninit<T>>` to `Buffer<T>`
    /// # Safety
    /// This function has the same safety as [`MaybeUninit`](std::mem::MaybeUninit)'s `assume_init`
    #[inline(always)]
    pub unsafe fn assume_init (self) -> Buffer<T, C> {
        self.transmute()
    }
}

impl<T: Copy, C: Context> Buffer<T, C> {
    /// Reads the contents of the buffer.
    pub fn read<'scope, 'env, R: IntoRange> (&'env self, scope: &'scope Scope<'scope, 'env, C>, range: R, wait: WaitList) -> Result<ReadEvent<'scope, T>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut result = Vec::<T>::with_capacity(len);

        let dst = Vec::as_mut_ptr(&mut result);
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(range, dst.cast(), queue, wait)
        };

        return scope.enqueue(supplier, BufferRead(result, PhantomData))
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
    pub fn read_into<'scope, 'env, R: IntoRange> (&'env self, s: &'scope Scope<'scope, 'env, C>, offset: usize, dst: &'env mut [T], wait: WaitList) -> Result<ReadIntoEvent<'scope, T, C>> {
        let range = BufferRange::from_parts::<T>(offset, dst.len())?;
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(range, dst.as_mut_ptr().cast(), queue, wait)
        };

        return s.enqueue_phantom(supplier)
    }

    /// Reads the contents of the buffer into `dst`, blocking the current thread until the operation has completed.
    #[inline]
    pub fn read_into_blocking<R: IntoRange> (&self, offset: usize, dst: &mut [T], wait: WaitList) -> Result<()> {
        let range = BufferRange::from_parts::<T>(offset, dst.len())?;
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(range, dst.as_mut_ptr().cast(), queue, wait)
        };

        self.ctx.next_queue().enqueue_noop(supplier)?.join()
    }

    /// Writes the contents of `src` into the buffer
    #[inline]
    pub fn write<'scope, 'env> (&'env mut self, scope: &'scope Scope<'scope, 'env, C>, offset: usize, src: &'env [T], wait: WaitList) -> Result<WriteEvent<'scope, T, C>> {
        let range = BufferRange::from_parts::<T>(offset, src.len()).unwrap();
        let supplier = |queue| unsafe {
            self.inner.write_from_ptr_in(range, src.as_ptr().cast(), queue, wait)
        };

        scope.enqueue_phantom(supplier)
    }

    /// Writes the contents of `src` into the buffer, blocking the current thread until the operation has completed.
    #[inline]
    pub fn write_blocking (&mut self, offset: usize, src: &[T], wait: WaitList) -> Result<()> {
        let range = BufferRange::from_parts::<T>(offset, src.len()).unwrap();
        let supplier = |queue| unsafe {
            self.inner.write_from_ptr_in(range, src.as_ptr().cast(), queue, wait)
        };

        self.ctx.next_queue().enqueue_noop(supplier)?.join()
    }

    /// Copies the contents from `self` to `dst`
    #[inline]
    pub fn copy_to<'scope, 'env> (&'env self, s: &'scope Scope<'scope, 'env, C>, src_offset: usize, dst: &'env mut Self, dst_offset: usize, size: usize, wait: WaitList) -> Result<CopyEvent<'scope, T, C>> {
        let src_offset = src_offset.checked_mul(core::mem::size_of::<T>()).unwrap();
        let dst_offset = dst_offset.checked_mul(core::mem::size_of::<T>()).unwrap();
        let size = size.checked_mul(core::mem::size_of::<T>()).unwrap();
        let supplier = |queue| unsafe {
            dst.copy_from_in(dst_offset, &self, src_offset, size, queue, wait)
        };

        s.enqueue_phantom(supplier)
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

        self.ctx.next_queue().enqueue_noop(supplier)?.join()
    }

    /// Copies the contents from `src` to `self`
    #[inline(always)]
    pub fn copy_from<'scope, 'env> (&'env mut self, s: &'scope Scope<'scope, 'env, C>, dst_offset: usize, src: &'env Self, src_offset: usize, size: usize, wait: WaitList) -> Result<CopyEvent<'scope, T, C>> {
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
    pub fn fill<'scope, 'env, R: IntoRange> (&'env mut self, s: &'scope Scope<'scope, 'env, C>, v: T, range: R, wait: WaitList) -> Result<FillEvent<'scope, T, C>> {
        let range = range.into_range::<T>(&self.inner)?;
        let supplier = |queue| unsafe {
            self.inner.fill_raw_in(v, range, queue, wait)
        };
        
        s.enqueue_phantom(supplier)
    }

    /// Fills a region of the buffer with `v`, blocking the current thread until the operation has completed.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill_blocking<R: IntoRange> (&mut self, v: T, range: R, wait: WaitList) -> Result<()> {
        let range = range.into_range::<T>(&self.inner)?;
        let supplier = |queue| unsafe {
            self.inner.fill_raw_in(v, range, queue, wait)
        };
        
        self.ctx.next_queue().enqueue_noop(supplier)?.join()
    }
}

impl<T, C: Context> Buffer<T, C> {
    pub fn map<'scope, 'env, R: IntoRange> (&'env self, s: &'scope Scope<'scope, 'env, C>, range: R, wait: WaitList) -> Result<BufferMapEvent<'scope, 'env, T, C>> {
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

    pub fn map_blocking<'a, R: IntoRange> (&'a self, range: R, wait: WaitList) -> Result<MapGuard<'a, T, C>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();
        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt)
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop(supplier)?.join()?;
            let ptr = core::slice::from_raw_parts_mut(ptr.assume_init() as *mut T, len);
            let ptr = MapPtr::new(ptr, self.inner.clone().into(), &self.ctx);
            return Ok(MapGuard::new(ptr)) 
        }
    }

    pub fn map_mut<'scope, 'env, R: IntoRange> (&'env mut self, s: &'scope Scope<'scope, 'env, C>, range: R, wait: WaitList) -> Result<BufferMapMutEvent<'scope, 'env, T, C>> {
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

    pub fn map_mut_blocking<'a, R: IntoRange> (&'a mut self, range: R, wait: WaitList) -> Result<MapMutGuard<'a, T, C>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();
        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt)
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop(supplier)?.join()?;
            let ptr = core::slice::from_raw_parts_mut(ptr.assume_init() as *mut T, len);
            let ptr = MapPtr::new(ptr, self.inner.clone().into(), &self.ctx);
            return Ok(MapMutGuard::new(ptr)) 
        }
    }
}

impl<T, C: Context> Deref for Buffer<T, C> {
    type Target = RawBuffer;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, C: Context> DerefMut for Buffer<T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: PartialEq, C: Context> PartialEq for Buffer<T, C> {
    fn eq(&self, other: &Self) -> bool {
        if self.eq_buffer(other) {
            return true;
        }

        let [this, other] = local_scope(&self.ctx, |s| {
            let this = self.map(s, .., None)?;
            let other = other.map(s, .., None)?;
            Event::join_all_sized_blocking([this, other])
        }).unwrap();

        this.deref() == other.deref()
    }
}

impl<T: Debug, C: Context> Debug for Buffer<T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.map_blocking(.., None).map_err(|_| std::fmt::Error)?;
        Debug::fmt(&v, f)
    }
}

impl<T: Eq, C: Context> Eq for Buffer<T, C> {}

pub struct BufferRead<'a, T: Copy, C: Context = Global> (Vec<T>, PhantomData<&'a Buffer<T, C>>);

impl<'a, T: Copy> Consumer for BufferRead<'a, T> {
    type Output = Vec<T>;
    
    #[inline(always)]
    fn consume (mut self) -> Result<Vec<T>> {
        unsafe { self.0.set_len(self.0.capacity()); }
        Ok(self.0)
    }
}

impl<'a, T: Copy> Debug for BufferRead<'a, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferRead").finish_non_exhaustive()
    }
}

/// Creates a buffer with sensible defaults.
/// 
/// This macro has three forms:
/// 
/// - Create a [`Buffer`] containing a list of elements
/// 
/// ```rust
/// use blaze_rs::{buffer, prelude::*};
/// 
/// let r#macro: Result<Buffer<i32>> = buffer![1, 2, 3];
/// let expanded: Result<Buffer<i32>> = Buffer::new(&[1, 2, 3], MemAccess::READ_WRITE, false);
/// 
/// assert_eq!(r#macro, expanded);
/// ```
/// 
/// - Create a [`Buffer`] with a given element and size
/// 
/// ```rust
/// use blaze_rs::{buffer, prelude::*};
/// 
/// let r#macro: Result<Buffer<i32>> = buffer![1; 3];
/// let expanded: Result<Buffer<i32>> = Buffer::new(&vec![1; 3], MemAccess::READ_WRITE, false);
/// 
/// assert_eq!(r#macro, expanded);
/// # Ok::<(), Error>(())
/// ```
/// 
/// - Create a [`Buffer`] with a by-index constructor
/// 
/// ```rust
/// use blaze_rs::{buffer, prelude::*};
/// 
/// let r#macro: Buffer<i32> = buffer![|i| i as i32; 3]?;
/// let expanded: Buffer<i32> = {
///     let mut res = Buffer::new_uninit(3, MemAccess::READ_WRITE, false)?;
///     for (i, v) in res.map_mut_blocking(.., WaitList::None)?.iter_mut().enumerate() {
///         v.write(i as i32);
///     }
///     res
/// };
/// 
/// assert_eq!(r#macro, expanded);
/// # Ok::<(), Error>(())
/// ```
/// 
/// In particular, the by-index constructor facilitates the construction of Buffers of `!Copy` types.
/// 
/// ```rust
/// use blaze_rs::{buffer, prelude::*};
/// 
/// #[repr(C)]
/// struct NoCopyStruct {
///     lock: bool,
///     val: i32,
/// }
/// 
/// let values: Buffer<NoCopyStruct> = buffer![|i| NoCopyStruct { lock: false, val: i as i32 }; 5]?;
/// # Ok::<(), Error>(())
/// ```
#[macro_export]
macro_rules! buffer {
    ($($v:expr),+) => {
        $crate::buffer::Buffer::new(&[$($v),+], $crate::buffer::flags::MemAccess::READ_WRITE, false)
    };

    (|$i:ident| $v:expr; $len:expr) => {
        (|| {
            let mut __1_ = $crate::buffer::Buffer::new_uninit($len, $crate::buffer::flags::MemAccess::READ_WRITE, false)?;
            let mut __2_ = __1_.map_mut_blocking(.., $crate::WaitList::None)?;
            for ($i, __3_) in __2_.into_iter().enumerate() {
                __3_.write($v);
            }

            unsafe { Ok::<_, $crate::core::Error>(__1_.assume_init()) }
        })()
    };

    ($v:expr; $len:expr) => {{
        $crate::buffer::Buffer::new(&::std::vec![$v; $len], $crate::buffer::flags::MemAccess::READ_WRITE, false)
    }};
}