flat_mod!(host, read);

use std::{ptr::NonNull, ops::{Deref, DerefMut}, num::NonZeroUsize};
use crate::{prelude::*, event::WaitList, memobj::IntoSlice2D};
use super::{Buffer, flags::{MemFlags, MemAccess, HostPtr}};

/// Buffer that conatins a 2D rectangle.
pub struct BufferRect2D<T: Copy, C: Context = Global> {
    inner: Buffer<T, C>,
    width: NonZeroUsize,
    height: NonZeroUsize
}

impl<T: Copy> BufferRect2D<T> {
    #[inline(always)]
    pub fn new (v: &Rect2D<T>, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::new_in(Global, v, access, alloc)
    }

    #[inline(always)]
    pub unsafe fn uninit (width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::uninit_in(Global, width, height, access, alloc)
    }
    
    #[inline]
    pub unsafe fn create (width: usize, height: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(Global, width, height, flags, host_ptr)
    }
}

impl<T: Copy, C: Context> BufferRect2D<T, C> {
    /// Creates new rectangular buffer
    #[inline]
    pub fn new_in (ctx: C, v: &Rect2D<T>, access: MemAccess, alloc: bool) -> Result<Self> {
        let host = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, v.width(), v.height(), host, NonNull::new(v.as_ptr() as *mut _)) }
    }

    #[inline]
    pub unsafe fn uninit_in (ctx: C, width: usize, height: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        let host = MemFlags::new(access, HostPtr::new(alloc, false));
        Self::create_in(ctx, width, height, host, None)
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

impl<T: Copy + Unpin, C: Context> BufferRect2D<T, C> {
    #[inline(always)]
    pub fn read<'src> (&'src self, slice: impl IntoSlice2D, wait: impl Into<WaitList>) -> Result<ReadBufferRect2D<'src, T>> {
        let (buffer_row_pitch, buffer_slice_pitch) = self.row_and_slice_pitch();
        unsafe { ReadBufferRect2D::new(self, self.width.get(), self.height.get(), slice, Some(buffer_row_pitch), Some(buffer_slice_pitch), self.inner.ctx.next_queue(), wait) }
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