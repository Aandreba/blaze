use super::{map::*, BufferRange, IntoRange};
use crate::blaze_rs;
use crate::buffer::{
    flags::{HostPtr, MemAccess, MemFlags},
    RawBuffer,
};
use crate::core::*;
use crate::{
    context::{local_scope, Context, Global, Scope},
    memobj::MapPtr,
    prelude::Event,
    WaitList,
};
use blaze_proc::join_various_blocking;
use blaze_proc::*;
use events::*;
use std::{
    ffi::c_void,
    fmt::Debug,
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::Arc,
};

pub mod events {
    use crate::{blaze_rs, event::Consumer, prelude::*};
    use blaze_proc::*;
    use std::{fmt::Debug, marker::*, mem::MaybeUninit, sync::Arc};

    /// Consumer for [`ReadIntoEvent`]
    #[newtype(pub(super))]
    pub type BufferReadInto<'a, T, C: Context = Global> =
        PhantomData<(&'a Buffer<T, C>, &'a mut [T])>;
    /// Consumer for [`WriteEvent`]
    #[newtype(pub(super))]
    pub type BufferWrite<'a, T, C: Context = Global> = PhantomData<(&'a mut Buffer<T, C>, &'a [T])>;
    /// Consumer for [`CopyEvent`]
    #[newtype(pub(super))]
    pub type BufferCopy<'a, T, C: Context = Global> =
        PhantomData<(&'a mut Buffer<T, C>, &'a Buffer<T, C>)>;
    /// Consumer for [`FillEvent`]
    #[docfg(feature = "cl1_2")]
    #[newtype(pub(super))]
    pub type BufferFill<'a, T, C: Context = Global> = PhantomData<(&'a mut Buffer<T, C>, T)>;

    /// Event for [`Buffer::get`]
    pub type GetEvent<'a, T, C = Global> = Event<BufferGet<'a, T, C>>;
    /// Event for [`Buffer::read`]
    pub type ReadEvent<'a, T, C = Global> = Event<BufferRead<'a, T, C>>;
    /// Event for [`Buffer::read_into`]
    pub type ReadIntoEvent<'a, T, C = Global> = Event<BufferReadInto<'a, T, C>>;
    /// Event for [`Buffer::write`]
    pub type WriteEvent<'a, T, C = Global> = Event<BufferWrite<'a, T, C>>;
    /// Event for [`Buffer::copy_from`] and [`Buffer::copy_to`]
    pub type CopyEvent<'a, T, C = Global> = Event<BufferCopy<'a, T, C>>;
    #[docfg(feature = "cl1_2")]
    /// Event for [`Buffer::fill`]
    pub type FillEvent<'a, T, C = Global> = Event<BufferFill<'a, T, C>>;

    /// Consumer for [`GetEvent`]
    pub struct BufferGet<'a, T: Copy, C: Context = Global> {
        pub(super) v: Arc<MaybeUninit<T>>,
        pub(super) _phtm: PhantomData<&'a Buffer<T, C>>,
    }

    impl<'a, T: Copy, C: Context> Consumer for BufferGet<'a, T, C> {
        type Output = T;

        #[inline(always)]
        unsafe fn consume(mut self) -> Result<T> {
            // Optimistic lock
            return loop {
                match Arc::try_unwrap(self.v) {
                    Ok(x) => break Ok(x.assume_init()),
                    Err(e) => {
                        self.v = e;
                        core::hint::spin_loop()
                    }
                }
            };
        }
    }

    impl<'a, T: Copy> Debug for BufferGet<'a, T> {
        #[inline(always)]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BufferGet").finish_non_exhaustive()
        }
    }

    /// Consumer for [`ReadEvent`]
    pub struct BufferRead<'a, T: Copy, C: Context = Global> {
        pub(super) vec: Arc<Vec<T>>,
        pub(super) _phtm: PhantomData<&'a Buffer<T, C>>,
    }

    impl<'a, T: Copy, C: Context> Consumer for BufferRead<'a, T, C> {
        type Output = Vec<T>;

        #[inline(always)]
        unsafe fn consume(mut self) -> Result<Vec<T>> {
            // Optimistic lock
            loop {
                match Arc::try_unwrap(self.vec) {
                    Ok(mut x) => {
                        x.set_len(x.capacity());
                        return Ok(x);
                    }

                    Err(e) => {
                        self.vec = e;
                        core::hint::spin_loop()
                    }
                }
            }
        }
    }

    impl<'a, T: Copy> Debug for BufferRead<'a, T> {
        #[inline(always)]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BufferRead").finish_non_exhaustive()
        }
    }
}

#[doc = include_str!("../../docs/src/buffer/README.md")]
pub struct Buffer<T, C: Context = Global> {
    pub(super) inner: RawBuffer,
    pub(super) ctx: C,
    pub(super) phtm: PhantomData<T>,
}

impl<T> Buffer<T> {
    /// Creates a new buffer with the given values and flags.
    #[inline(always)]
    pub fn new(v: &[T], access: MemAccess, alloc: bool) -> Result<Self>
    where
        T: Copy,
    {
        Self::new_in(Global, v, access, alloc)
    }

    /// Creates a new uninitialized buffer with the given size and flags.
    #[inline(always)]
    pub fn new_uninit(
        len: usize,
        access: MemAccess,
        alloc: bool,
    ) -> Result<Buffer<MaybeUninit<T>>> {
        Self::new_uninit_in(Global, len, access, alloc)
    }

    /// Creates a new zero-filled, uninitialized buffer with the given size and flags.
    /// If using OpenCL 1.2 or higher, this uses the `fill` event. Otherwise, a regular `write` is used.
    #[inline(always)]
    pub fn new_zeroed(
        len: usize,
        access: MemAccess,
        alloc: bool,
    ) -> Result<Buffer<MaybeUninit<T>>> {
        Self::new_zeroed_in(Global, len, access, alloc)
    }

    /// Creates a new buffer with the given custom parameters.
    #[inline(always)]
    pub unsafe fn create(
        len: usize,
        flags: MemFlags,
        host_ptr: Option<NonNull<T>>,
    ) -> Result<Self> {
        Self::create_in(Global, len, flags, host_ptr)
    }
}

impl<T, C: Context> Buffer<T, C> {
    /// Creates a new buffer with the given values and flags.
    #[inline]
    pub fn new_in(ctx: C, v: &[T], access: MemAccess, alloc: bool) -> Result<Self>
    where
        T: Copy,
    {
        let flags = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, v.len(), flags, NonNull::new(v.as_ptr() as *mut _)) }
    }

    /// Creates a new uninitialized buffer with the given size and flags.
    #[inline(always)]
    pub fn new_uninit_in(
        ctx: C,
        len: usize,
        access: MemAccess,
        alloc: bool,
    ) -> Result<Buffer<MaybeUninit<T>, C>> {
        let host = MemFlags::new(access, HostPtr::new(alloc, false));
        unsafe { Buffer::create_in(ctx, len, host, None) }
    }

    /// Creates a new zero-filled, uninitialized buffer with the given size and flags.
    /// If using OpenCL 1.2 or higher, this uses the `fill` event. Otherwise, a regular `write` is used.
    #[inline(always)]
    pub fn new_zeroed_in(
        ctx: C,
        len: usize,
        access: MemAccess,
        alloc: bool,
    ) -> Result<Buffer<MaybeUninit<T>, C>> {
        let mut buffer = Self::new_uninit_in(ctx, len, access, alloc)?;
        #[cfg(feature = "cl1_2")]
        {
            let range = (..).into_range::<T>(&buffer)?;
            let supplier = |queue| unsafe {
                buffer
                    .inner
                    .fill_raw_in(MaybeUninit::<T>::zeroed(), range, queue, None)
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
                buffer
                    .inner
                    .write_from_ptr_in(range, v.as_ptr().cast(), queue, None)
            };

            buffer.ctx.next_queue().enqueue_noop(supplier)?.join()?;
        }
        return Ok(buffer);
    }

    /// Creates a new buffer with the given custom parameters.
    #[inline]
    pub unsafe fn create_in(
        ctx: C,
        len: usize,
        flags: MemFlags,
        host_ptr: Option<NonNull<T>>,
    ) -> Result<Self> {
        let size = len.checked_mul(core::mem::size_of::<T>()).unwrap();
        let inner = RawBuffer::new_in(ctx.as_raw(), size, flags, host_ptr.map(NonNull::cast))?;

        Ok(Self {
            inner,
            ctx,
            phtm: PhantomData,
        })
    }

    /// Number of elements inside the buffer
    #[inline(always)]
    pub fn len(&self) -> Result<usize> {
        self.size().map(|x| x / core::mem::size_of::<T>())
    }

    /// Returns a reference to the [`Buffer`]'s underlying [`Context`].
    #[inline(always)]
    pub fn context(&self) -> &C {
        &self.ctx
    }

    /// Creates a shared slice of this buffer.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn slice<R: IntoRange>(&self, range: R) -> Result<super::Buf<'_, T, C>> {
        super::Buf::new(self, range)
    }

    /// Creates a mutable slice of this buffer.
    #[docfg(feature = "cl1_1")]
    #[inline(always)]
    pub fn slice_mut<R: IntoRange>(&mut self, range: R) -> Result<super::BufMut<'_, T, C>> {
        super::BufMut::new(self, range)
    }

    /// Reinterprets the bits of the buffer to another type.
    /// # Safety
    /// This function has the same safety as [`transmute`](std::mem::transmute)
    #[inline(always)]
    pub unsafe fn transmute<U>(self) -> Buffer<U, C> {
        debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());
        Buffer {
            inner: self.inner,
            ctx: self.ctx,
            phtm: PhantomData,
        }
    }

    /// Converts `Buffer<T,C>` into `Buffer<MaybeUninit<T>,C>`
    #[inline(always)]
    pub fn into_uninit(self) -> Buffer<MaybeUninit<T>, C> {
        unsafe { self.transmute() }
    }

    /// Converts `&Buffer<T,C>` to `&Buffer<MaybeUninit<T>,C>`
    #[inline(always)]
    pub const fn as_uninit(&self) -> &Buffer<MaybeUninit<T>, C> {
        unsafe { transmute(self) }
    }

    /// Converts `&mut Buffer<T,C>` to `&mut Buffer<MaybeUninit<T>,C>`
    #[inline(always)]
    pub fn as_mut_uninit(&mut self) -> &mut Buffer<MaybeUninit<T>, C> {
        unsafe { transmute(self) }
    }

    pub fn try_clone(&self, wait: WaitList) -> Result<Self>
    where
        T: Clone,
        C: Clone,
    {
        let len = self.len()?;
        let flags = self.flags()?;
        let mut result =
            Self::new_uninit_in(self.ctx.clone(), len, flags.access, flags.host.is_alloc())?;

        local_scope(self.context(), |s| {
            let (this, mut other): (MapGuard<_, _>, MapMutGuard<_, _>) =
                join_various_blocking!(self.map(s, .., wait)?, result.map_mut(s, .., wait)?)?;

            other
                .iter_mut()
                .zip(this.iter().cloned())
                .for_each(|(this, other)| {
                    this.write(other);
                });

            Ok(())
        })?;

        return unsafe { Ok(result.assume_init()) };
    }

    #[docfg(feature = "futures")]
    pub async fn try_clone_async<'a>(&self, wait: WaitList<'a>) -> Result<Self>
    where
        T: Clone,
        C: Clone,
    {
        let len = self.len()?;
        let flags = self.flags()?;
        let mut result =
            Self::new_uninit_in(self.ctx.clone(), len, flags.access, flags.host.is_alloc())?;

        crate::scope_async!(self.context(), |s| async {
            let (this, mut other): (MapGuard<_, _>, MapMutGuard<_, _>) = futures::try_join!(
                self.map(s, .., wait)?.join_async()?,
                result.map_mut(s, .., wait)?.join_async()?
            )?;

            other
                .iter_mut()
                .zip(this.iter().cloned())
                .for_each(|(this, other)| {
                    this.write(other);
                });

            Ok(())
        })
        .await?;

        return unsafe { Ok(result.assume_init()) };
    }

    /// Checks if the buffer pointer is the same in both buffers.
    #[inline(always)]
    pub fn eq_buffer(&self, other: &Buffer<T, C>) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T: 'static + Copy + Send + Sync, C: Context> Buffer<MaybeUninit<T>, C> {
    /// Convenience method for writing to an unitialized buffer. See [`write`](Buffer::write).
    #[inline(always)]
    pub fn write_init<'scope, 'env, O: Into<Option<usize>>>(
        &'env mut self,
        scope: &'scope Scope<'scope, 'env, C>,
        offset: O,
        src: &'env [T],
        wait: WaitList,
    ) -> Result<WriteEvent<'scope, MaybeUninit<T>, C>> {
        let src = unsafe { transmute::<&'env [T], &'env [MaybeUninit<T>]>(src) };
        self.write(scope, offset, src, wait)
    }

    /// Convenience method for writing to an unitialized buffer. See [`write_blocking`](Buffer::write_blocking).
    #[inline(always)]
    pub fn write_init_blocking(
        &mut self,
        offset: impl Into<Option<usize>>,
        src: &[T],
        wait: WaitList,
    ) -> Result<()> {
        let src = unsafe { transmute::<&[T], &[MaybeUninit<T>]>(src) };
        self.write_blocking(offset, src, wait)
    }

    /// Convenience method for copying to an unitialized buffer. See [`copy_from`](Buffer::copy_from).
    #[inline(always)]
    pub fn copy_from_init<
        'scope,
        'env,
        Dst: Into<Option<usize>>,
        Src: Into<Option<usize>>,
        Size: Into<Option<usize>>,
    >(
        &'env mut self,
        scope: &'scope Scope<'scope, 'env, C>,
        dst_offset: Dst,
        src: &'env Buffer<T, C>,
        src_offset: Src,
        size: Size,
        wait: WaitList,
    ) -> Result<CopyEvent<'scope, MaybeUninit<T>, C>> {
        unsafe { self.copy_from(scope, dst_offset, transmute(src), src_offset, size, wait) }
    }

    /// Convenience method for copying to an unitialized buffer. See [`copy_from_blocking`](Buffer::copy_from_blocking).
    #[inline(always)]
    pub fn copy_from_init_blocking(
        &mut self,
        dst_offset: impl Into<Option<usize>>,
        src: &Buffer<T, C>,
        src_offset: impl Into<Option<usize>>,
        size: impl Into<Option<usize>>,
        wait: WaitList,
    ) -> Result<()>
    where
        T: Copy + Send + Sync,
    {
        unsafe { self.copy_from_blocking(dst_offset, transmute(src), src_offset, size, wait) }
    }

    /// Convenience method for filling an unitialized buffer. See [`fill`](Buffer::fill).
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill_init<'scope, 'env, R: IntoRange>(
        &'env mut self,
        scope: &'scope Scope<'scope, 'env, C>,
        v: T,
        range: R,
        wait: WaitList,
    ) -> Result<FillEvent<'scope, MaybeUninit<T>, C>> {
        self.fill(scope, MaybeUninit::new(v), range, wait)
    }

    /// Convenience method for filling an unitialized buffer. See [`fill_blocking`](Buffer::fill_blocking).
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill_init_blocking(
        &mut self,
        v: T,
        range: impl IntoRange,
        wait: WaitList,
    ) -> Result<()> {
        self.fill_blocking(MaybeUninit::new(v), range, wait)
    }
}

impl<T, C: Context> Buffer<MaybeUninit<T>, C> {
    /// Extracts the value from `Buffer<MaybeUninit<T>>` to `Buffer<T>`
    /// # Safety
    /// This function has the same safety as [`MaybeUninit`](std::mem::MaybeUninit)'s `assume_init`
    #[inline(always)]
    pub unsafe fn assume_init(self) -> Buffer<T, C> {
        self.transmute()
    }
}

impl<T: 'static + Copy + Send + Sync, C: Context> Buffer<T, C> {
    /// Reads the contents of the buffer at the specified index, blocking the current thread until the operation has completed.
    pub fn get_blocking(&self, idx: usize, wait: WaitList) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(
                BufferRange::new(idx * core::mem::size_of::<T>(), core::mem::size_of::<T>()),
                result.as_mut_ptr().cast(),
                queue,
                wait,
            )
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop(supplier)?.join()?;
            return Ok(result.assume_init());
        }
    }

    /// Reads the contents of the buffer at the specified index, blocking the current thread until the operation has completed.
    pub fn get<'scope, 'env>(
        &'env self,
        scope: &'scope Scope<'scope, 'env, C>,
        idx: usize,
        wait: WaitList,
    ) -> Result<GetEvent<'scope, T, C>>
    where
        T: 'static + Send,
    {
        let result;
        cfg_if::cfg_if! {
            if #[cfg(feature = "nightly")] {
                result = Arc::<T>::new_uninit();
            } else{
                result = Arc::<MaybeUninit<T>>::new(MaybeUninit::uninit());
            }
        }

        let weak = Arc::downgrade(&result);
        let supplier = |queue| unsafe {
            self.inner.read_to_ptr_in(
                BufferRange::new(idx * core::mem::size_of::<T>(), core::mem::size_of::<T>()),
                result.as_ptr() as *mut c_void,
                queue,
                wait,
            )
        };

        let evt = scope.enqueue_noop(supplier)?.set_consumer(BufferGet {
            v: result,
            _phtm: PhantomData,
        });

        evt.on_complete_silent(move |_, _| drop(weak))?;
        return Ok(evt);
    }

    /// Reads the contents of the buffer.
    pub fn read<'scope, 'env, R: IntoRange>(
        &'env self,
        scope: &'scope Scope<'scope, 'env, C>,
        range: R,
        wait: WaitList,
    ) -> Result<ReadEvent<'scope, T, C>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();

        let mut result = Vec::<T>::with_capacity(len);
        let dst = Vec::as_mut_ptr(&mut result);

        let vec = Arc::new(result);
        let weak = Arc::downgrade(&vec);

        let supplier = |queue| unsafe { self.inner.read_to_ptr_in(range, dst.cast(), queue, wait) };

        let evt = scope.enqueue(
            supplier,
            BufferRead {
                vec,
                _phtm: PhantomData,
            },
        )?;

        evt.on_complete_silent(move |_, _| drop(weak))?;
        return Ok(evt);
    }

    /// Reads the contents of the buffer, blocking the current thread until the operation has completed.
    pub fn read_blocking<R: IntoRange>(&self, range: R, wait: WaitList) -> Result<Vec<T>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut result = Vec::<T>::with_capacity(len);

        let dst = Vec::as_mut_ptr(&mut result);
        let supplier = |queue| unsafe { self.inner.read_to_ptr_in(range, dst.cast(), queue, wait) };

        let f = move || unsafe {
            result.set_len(len);
            Ok(result)
        };

        unsafe { self.ctx.next_queue().enqueue_unchecked(supplier, f)?.join() }
    }

    /// Reads the contents of the buffer into `dst`.
    #[inline]
    pub fn read_into<'scope, 'env, O: Into<Option<usize>>>(
        &'env self,
        s: &'scope Scope<'scope, 'env, C>,
        offset: O,
        dst: &'env mut [T],
        wait: WaitList,
    ) -> Result<ReadIntoEvent<'scope, T, C>> {
        let range = BufferRange::from_parts::<T>(offset.into().unwrap_or_default(), dst.len())?;
        let supplier = |queue| unsafe {
            self.inner
                .read_to_ptr_in(range, dst.as_mut_ptr().cast(), queue, wait)
        };

        return Ok(Event::map_consumer(
            s.enqueue_phantom(supplier)?,
            BufferReadInto,
        ));
    }

    /// Reads the contents of the buffer into `dst`, blocking the current thread until the operation has completed.
    #[inline]
    pub fn read_into_blocking(
        &self,
        offset: impl Into<Option<usize>>,
        dst: &mut [T],
        wait: WaitList,
    ) -> Result<()> {
        let range = BufferRange::from_parts::<T>(offset.into().unwrap_or_default(), dst.len())?;
        let supplier = |queue| unsafe {
            self.inner
                .read_to_ptr_in(range, dst.as_mut_ptr().cast(), queue, wait)
        };

        self.ctx.next_queue().enqueue_noop(supplier)?.join()
    }

    /// Writes the contents of `src` into the buffer
    #[inline]
    pub fn write<'scope, 'env, O: Into<Option<usize>>>(
        &'env mut self,
        scope: &'scope Scope<'scope, 'env, C>,
        offset: O,
        src: &'env [T],
        wait: WaitList,
    ) -> Result<WriteEvent<'scope, T, C>> {
        let range =
            BufferRange::from_parts::<T>(offset.into().unwrap_or_default(), src.len()).unwrap();
        let supplier = |queue| unsafe {
            self.inner
                .write_from_ptr_in(range, src.as_ptr().cast(), queue, wait)
        };

        return Ok(Event::map_consumer(
            scope.enqueue_phantom(supplier)?,
            BufferWrite,
        ));
    }

    /// Writes the contents of `src` into the buffer, blocking the current thread until the operation has completed.
    #[inline]
    pub fn write_blocking(
        &mut self,
        offset: impl Into<Option<usize>>,
        src: &[T],
        wait: WaitList,
    ) -> Result<()> {
        let range =
            BufferRange::from_parts::<T>(offset.into().unwrap_or_default(), src.len()).unwrap();
        let supplier = |queue| unsafe {
            self.inner
                .write_from_ptr_in(range, src.as_ptr().cast(), queue, wait)
        };

        self.ctx.next_queue().enqueue_noop(supplier)?.join()
    }

    /// Copies the contents from `self` to `dst`
    #[inline]
    pub fn copy_to<
        'scope,
        'env,
        Src: Into<Option<usize>>,
        Dst: Into<Option<usize>>,
        Size: Into<Option<usize>>,
    >(
        &'env self,
        scope: &'scope Scope<'scope, 'env, C>,
        src_offset: Src,
        dst: &'env mut Self,
        dst_offset: Dst,
        size: Size,
        wait: WaitList,
    ) -> Result<CopyEvent<'scope, T, C>> {
        let src_offset = src_offset
            .into()
            .unwrap_or_default()
            .checked_mul(core::mem::size_of::<T>())
            .ok_or_else(|| Error::from_type(ErrorKind::InvalidValue))?;
        let dst_offset = dst_offset
            .into()
            .unwrap_or_default()
            .checked_mul(core::mem::size_of::<T>())
            .ok_or_else(|| Error::from_type(ErrorKind::InvalidValue))?;
        let size = match size.into() {
            Some(x) => x
                .checked_mul(core::mem::size_of::<T>())
                .ok_or_else(|| Error::from_type(ErrorKind::InvalidValue))?,
            None => self.size()? - src_offset,
        };

        let supplier =
            |queue| unsafe { dst.copy_from_in(dst_offset, &self, src_offset, size, queue, wait) };

        return Ok(Event::map_consumer(
            scope.enqueue_phantom(supplier)?,
            BufferCopy,
        ));
    }

    /// Copies the contents from `self` to `dst`, blocking the current thread until the operation has completed.
    #[inline]
    pub fn copy_to_blocking(
        &self,
        src_offset: impl Into<Option<usize>>,
        dst: &mut Self,
        dst_offset: impl Into<Option<usize>>,
        size: impl Into<Option<usize>>,
        wait: WaitList,
    ) -> Result<()> {
        let src_offset = src_offset
            .into()
            .unwrap_or_default()
            .checked_mul(core::mem::size_of::<T>())
            .ok_or_else(|| Error::from_type(ErrorKind::InvalidValue))?;
        let dst_offset = dst_offset
            .into()
            .unwrap_or_default()
            .checked_mul(core::mem::size_of::<T>())
            .ok_or_else(|| Error::from_type(ErrorKind::InvalidValue))?;
        let size = match size.into() {
            Some(x) => x
                .checked_mul(core::mem::size_of::<T>())
                .ok_or_else(|| Error::from_type(ErrorKind::InvalidValue))?,
            None => self.size()? - src_offset,
        };

        let supplier =
            |queue| unsafe { dst.copy_from_in(dst_offset, &self, src_offset, size, queue, wait) };

        self.ctx.next_queue().enqueue_noop(supplier)?.join()
    }

    /// Copies the contents from `src` to `self`
    #[inline(always)]
    pub fn copy_from<
        'scope,
        'env,
        Dst: Into<Option<usize>>,
        Src: Into<Option<usize>>,
        Size: Into<Option<usize>>,
    >(
        &'env mut self,
        s: &'scope Scope<'scope, 'env, C>,
        dst_offset: Dst,
        src: &'env Self,
        src_offset: Src,
        size: Size,
        wait: WaitList,
    ) -> Result<CopyEvent<'scope, T, C>> {
        src.copy_to(s, src_offset, self, dst_offset, size, wait)
    }

    /// Copies the contents from `src` to `self`, blocking the current thread until the operation has completed.
    #[inline(always)]
    pub fn copy_from_blocking(
        &mut self,
        dst_offset: impl Into<Option<usize>>,
        src: &Self,
        src_offset: impl Into<Option<usize>>,
        size: impl Into<Option<usize>>,
        wait: WaitList,
    ) -> Result<()> {
        src.copy_to_blocking(src_offset, self, dst_offset, size, wait)
    }

    /// Fills a region of the buffer with `v`
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill<'scope, 'env, R: IntoRange>(
        &'env mut self,
        scope: &'scope Scope<'scope, 'env, C>,
        v: T,
        range: R,
        wait: WaitList,
    ) -> Result<FillEvent<'scope, T, C>> {
        let range = range.into_range::<T>(&self.inner)?;
        let supplier = |queue| unsafe { self.inner.fill_raw_in(v, range, queue, wait) };

        return Ok(Event::map_consumer(
            scope.enqueue_phantom(supplier)?,
            BufferFill,
        ));
    }

    /// Fills a region of the buffer with `v`, blocking the current thread until the operation has completed.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn fill_blocking<R: IntoRange>(&mut self, v: T, range: R, wait: WaitList) -> Result<()> {
        let range = range.into_range::<T>(&self.inner)?;
        let supplier = |queue| unsafe { self.inner.fill_raw_in(v, range, queue, wait) };

        self.ctx.next_queue().enqueue_noop(supplier)?.join()
    }
}

impl<T, C: Context> Buffer<T, C> {
    pub fn map<'scope, 'env, R: IntoRange>(
        &'env self,
        s: &'scope Scope<'scope, 'env, C>,
        range: R,
        wait: WaitList,
    ) -> Result<BufferMapEvent<'scope, 'env, T, C>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();

        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt);
        };

        unsafe {
            let noop = s.enqueue_noop(supplier)?;
            let consumer = BufferMap::new(ptr.assume_init(), self, len);
            return Ok(noop.set_consumer(consumer));
        }
    }

    pub fn map_blocking<'a, R: IntoRange>(
        &'a self,
        range: R,
        wait: WaitList,
    ) -> Result<MapGuard<'a, T, C>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();
        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt);
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop(supplier)?.join()?;
            let ptr = core::slice::from_raw_parts_mut(ptr.assume_init() as *mut T, len);
            let ptr = MapPtr::new(ptr, self.inner.clone().into(), &self.ctx);
            return Ok(MapGuard::new(ptr));
        }
    }

    pub fn map_mut<'scope, 'env, R: IntoRange>(
        &'env mut self,
        s: &'scope Scope<'scope, 'env, C>,
        range: R,
        wait: WaitList,
    ) -> Result<BufferMapMutEvent<'scope, 'env, T, C>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();

        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt);
        };

        unsafe {
            let noop = s.enqueue_noop(supplier)?;
            let consumer = BufferMapMut::new(ptr.assume_init(), self, len);
            return Ok(noop.set_consumer(consumer));
        }
    }

    pub fn map_mut_blocking<'a, R: IntoRange>(
        &'a mut self,
        range: R,
        wait: WaitList,
    ) -> Result<MapMutGuard<'a, T, C>> {
        let range = range.into_range::<T>(&self.inner)?;
        let len = range.cb / core::mem::size_of::<T>();
        let mut ptr = MaybeUninit::uninit();
        let supplier = |queue| unsafe {
            let (_ptr, evt) = self.inner.map_read_in(range, queue, wait)?;
            ptr.write(_ptr);
            return Ok(evt);
        };

        unsafe {
            self.ctx.next_queue().enqueue_noop(supplier)?.join()?;
            let ptr = core::slice::from_raw_parts_mut(ptr.assume_init() as *mut T, len);
            let ptr = MapPtr::new(ptr, self.inner.clone().into(), &self.ctx);
            return Ok(MapMutGuard::new(ptr));
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

impl<T: Clone, C: Context + Clone> Clone for Buffer<T, C> {
    #[inline(always)]
    fn clone(&self) -> Self {
        self.try_clone(None).unwrap()
    }

    fn clone_from(&mut self, source: &Self) {
        let len = self.len().unwrap();
        assert_eq!(len, source.len().unwrap());

        local_scope(source.context(), |s| {
            let (this, mut other) =
                join_various_blocking!(source.map(s, .., None)?, self.map_mut(s, .., None)?)?;

            other
                .iter_mut()
                .zip(this.iter().cloned())
                .for_each(|(other, this)| *other = this);

            Ok(())
        })
        .unwrap();
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

            #[cfg(feature = "nightly")]
            return Event::join_sized_blocking([this, other]);
            #[cfg(not(feature = "nightly"))]
            {
                let vec = Event::join_all_blocking([this, other])?;
                match <[MapGuard<T, C>; 2] as TryFrom<Vec<MapGuard<T, C>>>>::try_from(vec) {
                    Ok(x) => Ok(x),
                    Err(_) => unsafe { core::hint::unreachable_unchecked() },
                }
            }
        })
        .unwrap();

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

/// Creates a buffer with sensible defaults.
///
/// This macro has three forms:
///
/// - Create a [`Buffer`] containing a list of elements
///
/// ```rust
/// use blaze_rs::{buffer, prelude::*};
///
/// #[global_context]
/// static CONTEXT : SimpleContext = SimpleContext::default();
///
/// let r#macro: Result<Buffer<i32>> = buffer![1, 2, 3];
/// let expanded: Result<Buffer<i32>> = Buffer::new(&[1, 2, 3], MemAccess::READ_WRITE, false);
/// ```
///
/// - Create a [`Buffer`] with a given element and size
///
/// ```rust
/// use blaze_rs::{buffer, prelude::*};
///
/// #[global_context]
/// static CONTEXT : SimpleContext = SimpleContext::default();
///
/// let r#macro: Result<Buffer<i32>> = buffer![1; 3];
/// let expanded: Result<Buffer<i32>> = Buffer::new(&vec![1; 3], MemAccess::READ_WRITE, false);
/// ```
///
/// - Create a [`Buffer`] with a by-index constructor
///
/// ```rust
/// use blaze_rs::{buffer, prelude::*};
///
/// #[global_context]
/// static CONTEXT : SimpleContext = SimpleContext::default();
///
/// # fn main () -> Result<()> {
///
/// let r#macro: Buffer<i32> = buffer![|i| i as i32; 3]?;
/// let expanded: Buffer<i32> = unsafe {
///     let mut res = Buffer::new_uninit(3, MemAccess::READ_WRITE, false)?;
///     for (i, v) in res.map_mut_blocking(.., WaitList::None)?.iter_mut().enumerate() {
///         v.write(i as i32);
///     }
///     res.assume_init()
/// };
///
/// # Ok::<(), Error>(())
/// # }
/// ```
///
/// In particular, the by-index constructor facilitates the construction of Buffers of `!Copy` types.
///
/// ```rust
/// use blaze_rs::{buffer, prelude::*};
///
/// #[global_context]
/// static CONTEXT : SimpleContext = SimpleContext::default();
///
/// #[repr(C)]
/// struct NoCopyStruct {
///     lock: bool,
///     val: i32,
/// }
///
/// let values: Result<Buffer<NoCopyStruct>> = buffer![|i| NoCopyStruct { lock: false, val: i as i32 }; 5];
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
