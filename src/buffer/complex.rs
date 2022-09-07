use std::{marker::PhantomData, ptr::{NonNull}, ops::{Deref, DerefMut}, fmt::Debug, mem::MaybeUninit};
//use blaze_proc::docfg;

use crate::{context::{Context, Global, LocalScope}, prelude::{Event, RawEvent, scope::local_scope}};
use crate::core::*;
use crate::buffer::{flags::{MemFlags, HostPtr, MemAccess}, RawBuffer};
use super::{IntoRange, BufferRange};

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
        //buffer.fill(MaybeUninit::zeroed(), .., WaitList::EMPTY)?.wait()?;
        todo!();
        #[cfg(not(feature = "cl1_2"))]
        todo!();
        //buffer.write(0, vec![MaybeUninit::zeroed(); len], WaitList::EMPTY)?.wait()?;
        
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
    pub fn read_blocking<R: IntoRange> (&self, range: R, wait: &[RawEvent]) -> Result<Vec<T>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut result = Vec::<T>::with_capacity(len);

        let dst = Vec::as_mut_ptr(&mut result);
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(range, dst, queue, wait)
        };

        let f = move || unsafe {
            result.set_len(len);
            Ok(result)
        };

        unsafe {
            self.ctx.next_queue().enqueue_unchecked(supplier, f)?.join()
        }
    }

    pub fn read<'ctx, 'scope, 'env, R: IntoRange> (&'scope self, scope: &'scope LocalScope<'ctx, 'scope, 'env>, range: R, wait: &[RawEvent]) -> Result<Event<'_, Vec<T>>> where C: 'ctx, T: 'scope {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut result = Vec::<T>::with_capacity(len);
        
        let dst = Vec::as_mut_ptr(&mut result);
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(range, dst, queue, wait)
        };

        let f = move || unsafe {
            //let _ = self;
            result.set_len(len);
            Ok(result)
        };

        return scope.enqueue(supplier, f)
    }

    pub fn write<'ctx, 'scope, 'env> (&'scope mut self, scope: &'scope LocalScope<'ctx, 'scope, 'env>, offset: usize, src: &'env [T], wait: &[RawEvent]) -> Result<Event<'scope, ()>> {
        let range = BufferRange::from_parts::<T>(offset, src.len()).unwrap();
        let supplier = |queue| unsafe {
            self.inner.write_from_ptr_in(range, src.as_ptr(), queue, wait)
        };

        scope.enqueue_noop(supplier)
    }

    pub fn write_blocking (&mut self, offset: usize, src: &[T], wait: &[RawEvent]) -> Result<()> {
        let range = BufferRange::from_parts::<T>(offset, src.len()).unwrap();
        let supplier = |queue| unsafe {
            self.inner.write_from_ptr_in(range, src.as_ptr(), queue, wait)
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop_unchecked(supplier)?.join()
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
        local_scope(&self.ctx, |s| {
            let this = self.read(s, .., &[])?;
            let other = other.read(s, .., &[])?;
            todo!()
        });

        todo!()

        /*let this = match self.read(.., &[]) {
            Ok(x) => x,
            Err(_) => return false
        };

        let other = match other.read(.., &[]) {
            Ok(x) => x,
            Err(_) => return false
        };
        
        let join = match ReadBuffer::join_blocking([this, other]) {
            Ok(x) => x,
            Err(_) => return false
        };

        join[0] == join[1]*/
    }
}

impl<T: Copy + Unpin + Debug, C: Context> Debug for Buffer<T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.read_blocking(.., &[]).unwrap();
        Debug::fmt(&v, f)
    }
}

impl<T: Copy + Unpin + Eq, C: Context> Eq for Buffer<T, C> {}