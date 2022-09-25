flat_mod!(host);

use std::{ptr::NonNull, ops::{Deref, DerefMut}, num::NonZeroUsize, mem::MaybeUninit, fmt::Debug, marker::PhantomData};
use crate::{prelude::*, event::Consumer};
use super::{Buffer, flags::{MemFlags, MemAccess, HostPtr}};
use blaze_proc::docfg;

#[deprecated(since = "0.1.0", note = "use `RectBuffer2D` instead")]
pub type BufferRect2D<T, C = Global> = RectBuffer2D<T, C>;
pub type ReadRectEvent<'a, T, C = Global> = Event<ReadRect<'a, T, C>>;

/// Buffer that conatins a 2D rectangle.
pub struct RectBuffer2D<T, C: Context = Global> {
    inner: Buffer<T, C>,
    width: NonZeroUsize,
    height: NonZeroUsize
}

impl<T> RectBuffer2D<T> {
    /// Creates a new rectangular buffer from the specified values in [row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).
    #[inline(always)]
    pub fn new (v: &[T], width: usize, access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
        Self::new_in(Global, v, width, access, alloc)
    }

    #[inline(always)]
    pub fn from_rect (v: &RectBox2D<T>, access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
        Self::from_rect_in(Global, v, access, alloc)
    }

    #[inline(always)]
    pub fn new_uninit (width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<RectBuffer2D<MaybeUninit<T>>> {
        Self::new_uninit_in(Global, width, height, access, alloc)
    }
    
    #[inline]
    pub unsafe fn create (width: usize, height: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(Global, width, height, flags, host_ptr)
    }
}

impl<T, C: Context> RectBuffer2D<T, C> {
    /// Creates a new rectangular buffer, in the specified context, from the specified values in [row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).
    #[inline]
    pub fn new_in (ctx: C, v: &[T], width: usize, access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
        let height = v.len() / width;
        let host = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, width, height, host, NonNull::new(v.as_ptr() as *mut _)) }
    }

    /// Creates new rectangular buffer
    #[inline]
    pub fn from_rect_in (ctx: C, v: &RectBox2D<T>, access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
        let host = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, v.width(), v.height(), host, NonNull::new(v.as_ptr() as *mut _)) }
    }

    #[inline]
    pub fn new_uninit_in (ctx: C, width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<RectBuffer2D<MaybeUninit<T>, C>> {
        let host = MemFlags::new(access, HostPtr::new(alloc, false));
        unsafe { RectBuffer2D::create_in(ctx, width, height, host, None) }
    }
    
    #[inline]
    pub unsafe fn create_in (ctx: C, width: usize, height: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        match width.checked_mul(height) {
            Some(0) | None => Err(Error::new(ErrorKind::InvalidBufferSize, "overflow multiplying 'rows' and 'cols'")),
            Some(len) => {
                let inner = Buffer::create_in(ctx, len, flags, host_ptr)?;
                let rows = NonZeroUsize::new_unchecked(width);
                let cols = NonZeroUsize::new_unchecked(height);
                Ok(Self { inner, width: rows, height: cols, })
            }
        }
    }

    #[inline(always)]
    pub fn as_flat (&self) -> &Buffer<T, C> {
        &self.inner
    }

    #[inline(always)]
    pub fn as_mut_flat (&mut self) -> &mut Buffer<T, C> {
        &mut self.inner
    }
    
    #[inline(always)]
    pub fn flatten (self) -> Buffer<T, C> {
        self.inner
    }

    #[inline(always)]
    pub unsafe fn transmute<U: Copy> (self) -> RectBuffer2D<U, C> {
        RectBuffer2D::<U, C> { inner: self.inner.transmute(), width: self.width, height: self.height }
    }
}

impl<T: Copy, C: Context> RectBuffer2D<MaybeUninit<T>, C> {
    #[inline(always)]
    pub unsafe fn assume_init (self) -> RectBuffer2D<T, C> {
        self.transmute()
    }
}

impl<T: Copy, C: Context> RectBuffer2D<T, C> {
    #[inline(always)]
    pub fn width (&self) -> usize {
        self.width.get()
    }

    #[inline(always)]
    pub fn height (&self) -> usize {
        self.height.get()
    }

    #[inline(always)]
    pub fn row_pitch (&self) -> usize {
        self.width() * core::mem::size_of::<T>()
    }

    #[inline(always)]
    pub fn slice_pitch (&self) -> usize {
        self.height() * self.row_pitch()
    }

    #[inline(always)]
    pub fn row_and_slice_pitch (&self) -> (usize, usize) {
        let row = self.row_pitch();
        (row, self.height() * row)
    }
}

#[cfg(feature = "cl1_1")]
use crate::{WaitList, memobj::IntoRange2D};

#[docfg(feature = "cl1_1")]
impl<T: Copy, C: Context> RectBuffer2D<T, C> {
    pub fn read<'scope, 'env, R: IntoRange2D> (&'env self, scope: &'scope Scope<'scope, 'env, C>, range: R, wait: WaitList) -> Result<ReadRectEvent<'scope, T, C>> {
        let (buffer_row_pitch, buffer_slice_pitch) = self.row_and_slice_pitch();
        let range = range.into_range(self.width(), self.height())?;

        let [buffer_origin, region] = range.raw_parts_buffer::<T>();
        let mut dst = RectBox2D::<T>::new_uninit(range.width(), range.height())
            .ok_or_else(|| Error::from_type(ErrorKind::InvalidBufferSize)
        )?;

        let supplier = |queue| unsafe {
            self.read_rect_to_ptr_in(
                buffer_origin, [0; 3], region,
                Some(buffer_row_pitch), Some(buffer_slice_pitch),
                Some(0), Some(0),
                dst.as_mut_ptr().cast(), queue, wait
            )
        };

        return Ok(scope
            .enqueue_noop(supplier)?
            .set_consumer(ReadRect(dst, PhantomData))
        )
    }
    
    pub fn read_blocking<R: IntoRange2D> (&self, range: R, wait: WaitList) -> Result<RectBox2D<T>> {
        let (buffer_row_pitch, buffer_slice_pitch) = self.row_and_slice_pitch();
        let range = range.into_range(self.width(), self.height())?;

        let [buffer_origin, region] = range.raw_parts_buffer::<T>();
        let mut dst = RectBox2D::<T>::new_uninit(range.width(), range.height())
            .ok_or_else(|| Error::from_type(ErrorKind::InvalidBufferSize)
        )?;

        let supplier = |queue| unsafe {
            self.read_rect_to_ptr_in(
                buffer_origin, [0; 3], region,
                Some(buffer_row_pitch), Some(buffer_slice_pitch),
                Some(0), Some(0),
                dst.as_mut_ptr().cast(), queue, wait
            )
        };

        self.context().next_queue().enqueue_noop(supplier)?.join()?;
        return unsafe { Ok(dst.assume_init()) }
    }

    pub fn write_blocking (&mut self, offset: [usize; 2], src: (&[T], usize), wait: WaitList) -> Result<()> {
        let (buffer_row_pitch, buffer_slice_pitch) = self.row_and_slice_pitch();
        let host_row_pitch = src.1 * core::mem::size_of::<T>();
        let buffer_origin = [offset_src[0] * core::mem::size_of::<T>(), offset_src[1], 0];
        let host_origin = [offset_dst[0] * core::mem::size_of::<T>(), offset_dst[1], 0];

        let supplier = |queue| unsafe {
            self.write_rect_from_ptr_in(
                buffer_origin, host_origin, region,
                buffer_row_pitch, buffer_slice_pitch,
                host_row_pitch, host_slice_pitch,
                src, queue, wait
            )
        };

        return self.context().next_queue().enqueue_noop(supplier)?.join()
    }
}

impl<T, C: Context> Deref for RectBuffer2D<T, C> {
    type Target = Buffer<T, C>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, C: Context> DerefMut for RectBuffer2D<T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: PartialEq, C: Context> PartialEq for RectBuffer2D<T, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && 
        self.height == other.height &&
        self.inner == other.inner
    }
}

impl<T: Debug, C: Context> Debug for RectBuffer2D<T, C> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map = Buffer::map_blocking(&self, .., None).map_err(|_| std::fmt::Error)?;
        f.debug_list().entries(map.chunks(self.width.get())).finish()
    }
}

impl<T: Eq, C: Context> Eq for RectBuffer2D<T, C> {}

pub struct ReadRect<'a, T: Copy, C: Context = Global> (RectBox2D<MaybeUninit<T>>, PhantomData<&'a RectBuffer2D<T, C>>);

impl<'a, T: Copy, C: Context> Consumer for ReadRect<'a, T, C> {
    type Output = RectBox2D<T>;

    #[inline(always)]
    fn consume (self) -> Result<Self::Output> {
        unsafe {
            Ok(self.0.assume_init())
        }
    }
}

impl<'a, T: Copy> Debug for ReadRect<'a, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadRect").finish_non_exhaustive()
    }
}