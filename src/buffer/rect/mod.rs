flat_mod!(host);
#[cfg(feature = "cl1_1")]
flat_mod!(read, write);

use std::{ptr::NonNull, ops::{Deref, DerefMut}, num::NonZeroUsize, mem::MaybeUninit, fmt::Debug};
use blaze_proc::docfg;
use crate::{prelude::*, event::WaitList};
use super::{Buffer, flags::{MemFlags, MemAccess, HostPtr}};

/// Buffer that conatins a 2D rectangle.
pub struct BufferRect2D<T: Copy, C: Context = Global> {
    inner: Buffer<T, C>,
    width: NonZeroUsize,
    height: NonZeroUsize
}

impl<T: Copy> BufferRect2D<T> {
    /// Creates a new rectangular buffer from the specified values in [row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).
    #[inline(always)]
    pub fn new (v: &[T], width: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::new_in(Global, v, width, access, alloc)
    }

    #[inline(always)]
    pub fn from_rect (v: &Rect2D<T>, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::from_rect_in(Global, v, access, alloc)
    }

    #[inline(always)]
    pub fn new_uninit (width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<BufferRect2D<MaybeUninit<T>>> {
        Self::new_uninit_in(Global, width, height, access, alloc)
    }
    
    #[inline]
    pub unsafe fn create (width: usize, height: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(Global, width, height, flags, host_ptr)
    }
}

impl<T: Copy, C: Context> BufferRect2D<T, C> {
    /// Creates a new rectangular buffer, in the specified context, from the specified values in [row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).
    #[inline]
    pub fn new_in (ctx: C, v: &[T], width: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        let height = v.len() / width;
        let host = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, width, height, host, NonNull::new(v.as_ptr() as *mut _)) }
    }

    /// Creates new rectangular buffer
    #[inline]
    pub fn from_rect_in (ctx: C, v: &Rect2D<T>, access: MemAccess, alloc: bool) -> Result<Self> {
        let host = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, v.width(), v.height(), host, NonNull::new(v.as_ptr() as *mut _)) }
    }

    #[inline]
    pub fn new_uninit_in (ctx: C, width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<BufferRect2D<MaybeUninit<T>, C>> {
        let host = MemFlags::new(access, HostPtr::new(alloc, false));
        unsafe { BufferRect2D::create_in(ctx, width, height, host, None) }
    }
    
    #[inline]
    pub unsafe fn create_in (ctx: C, width: usize, height: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        match width.checked_mul(height) {
            Some(0) | None => Err(Error::new(ErrorType::InvalidBufferSize, "overflow multiplying 'rows' and 'cols'")),
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
    pub unsafe fn transmute<U: Copy> (self) -> BufferRect2D<U, C> {
        BufferRect2D::<U, C> { inner: self.inner.transmute(), width: self.width, height: self.height }
    }
}

impl<T: Copy, C: Context> BufferRect2D<MaybeUninit<T>, C> {
    #[inline(always)]
    pub unsafe fn assume_init (self) -> BufferRect2D<T, C> {
        self.transmute()
    }
}

impl<T: Copy, C: Context> BufferRect2D<T, C> {
    #[inline(always)]
    pub fn width (&self) -> NonZeroUsize {
        self.width
    }

    #[inline(always)]
    pub fn height (&self) -> NonZeroUsize {
        self.height
    }

    #[inline(always)]
    pub fn row_pitch (&self) -> usize {
        self.width.get() * core::mem::size_of::<T>()
    }

    #[inline(always)]
    pub fn slice_pitch (&self) -> usize {
        self.height.get() * self.row_pitch()
    }

    #[inline(always)]
    pub fn row_and_slice_pitch (&self) -> (usize, usize) {
        let row = self.row_pitch();
        (row, self.height.get() * row)
    }
}

#[docfg(feature = "cl1_1")]
impl<T: Copy + Unpin, C: Context> BufferRect2D<T, C> {
    #[inline(always)]
    pub fn read_all<'src> (&'src self, wait: impl Into<WaitList>) -> Result<ReadBufferRect2D<'src, T>> {
        self.read((.., ..), wait)
    }

    #[inline(always)]
    pub fn read<'src> (&'src self, slice: impl crate::memobj::IntoSlice2D, wait: impl Into<WaitList>) -> Result<ReadBufferRect2D<'src, T>> {
        let (buffer_row_pitch, buffer_slice_pitch) = self.row_and_slice_pitch();
        unsafe { ReadBufferRect2D::new(self, self.width.get(), self.height.get(), slice, Some(buffer_row_pitch), Some(buffer_slice_pitch), self.inner.ctx.next_queue(), wait) }
    }

    #[inline(always)]
    pub fn read_into<'src, Dst: DerefMut<Target = Rect2D<T>>> (&'src self, offset_src: [usize; 2], dst: Dst, offset_dst: [usize; 2], region: [usize; 2], wait: impl Into<WaitList>) -> Result<ReadIntoBufferRect2D<&'src Self, Dst>> {
        Self::read_into_by_deref(self, offset_src, dst, offset_dst, region, wait)
    }

    #[inline(always)]
    pub fn write<'dst, Src: Deref<Target = Rect2D<T>>> (&'dst mut self, offset_dst: [usize; 2], src: Src, offset_src: [usize; 2], region: [usize; 2], wait: impl Into<WaitList>) -> Result<WriteBufferRect2D<Src, &'dst mut Self>> {
        Self::write_by_deref(self, offset_dst, src, offset_src, region, wait)
    }

    #[inline(always)]
    pub fn read_into_by_deref<Src: Deref<Target = Self>, Dst: DerefMut<Target = Rect2D<T>>> (this: Src, offset_src: [usize; 2], dst: Dst, offset_dst: [usize; 2], region: [usize; 2], wait: impl Into<WaitList>) -> Result<ReadIntoBufferRect2D<Src, Dst>> {
        let (buffer_row_pitch, buffer_slice_pitch) = this.row_and_slice_pitch();
        let queue = this.inner.ctx.next_queue().clone();
        unsafe { ReadIntoBufferRect2D::<Src, Dst>::new(this, offset_src, dst, offset_dst, region, Some(buffer_row_pitch), Some(buffer_slice_pitch), &queue, wait) }
    }

    #[inline(always)]
    pub fn write_by_deref<Dst: DerefMut<Target = Self>, Src: Deref<Target = Rect2D<T>>> (this: Dst, offset_dst: [usize; 2], src: Src, offset_src: [usize; 2], region: [usize; 2], wait: impl Into<WaitList>) -> Result<WriteBufferRect2D<Src, Dst>> {
        let (buffer_row_pitch, buffer_slice_pitch) = this.row_and_slice_pitch();
        let queue = this.inner.ctx.next_queue().clone();
        unsafe { WriteBufferRect2D::new(src, offset_src, this, offset_dst, region, Some(buffer_row_pitch), Some(buffer_slice_pitch), &queue, wait) }
    }
}

impl<T: Copy, C: Context> Deref for BufferRect2D<T, C> {
    type Target = Buffer<T, C>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Copy, C: Context> DerefMut for BufferRect2D<T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Copy + Unpin + Debug, C: Context> Debug for BufferRect2D<T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let all = self.read_all(WaitList::EMPTY).unwrap().wait().unwrap();
        Debug::fmt(&all, f)
    }
}