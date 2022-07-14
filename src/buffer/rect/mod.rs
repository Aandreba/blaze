flat_mod!(host, read);

use std::{ptr::NonNull, ops::{Deref, DerefMut}, num::NonZeroUsize};
use crate::{prelude::*, event::WaitList, memobj::IntoSlice2D};
use super::{Buffer, flags::{MemFlags, MemAccess, HostPtr}};

pub struct BufferRect2D<T: Copy, C: Context = Global> {
    inner: Buffer<T, C>,
    rows: NonZeroUsize,
    cols: NonZeroUsize
}

impl<T: Copy> BufferRect2D<T> {
    #[inline(always)]
    pub fn new (v: &[T], rows: usize, cols: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::new_in(Global, v, rows, cols, access, alloc)
    }

    #[inline(always)]
    pub unsafe fn uninit (rows: usize, cols: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        Self::uninit_in(Global, rows, cols, access, alloc)
    }
    
    #[inline]
    pub unsafe fn create (rows: usize, cols: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        Self::create_in(Global, rows, cols, flags, host_ptr)
    }
}

impl<T: Copy, C: Context> BufferRect2D<T, C> {
    #[inline]
    pub fn new_in (ctx: C, v: &[T], rows: usize, cols: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        assert_eq!(Some(v.len()), rows.checked_mul(cols));
        let host = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, rows, cols, host, NonNull::new(v.as_ptr() as *mut _)) }
    }

    #[inline]
    pub unsafe fn uninit_in (ctx: C, rows: usize, cols: usize, access: MemAccess, alloc: bool) -> Result<Self> {
        let host = MemFlags::new(access, HostPtr::new(alloc, false));
        Self::create_in(ctx, rows, cols, host, None)
    }
    
    #[inline]
    pub unsafe fn create_in (ctx: C, rows: usize, cols: usize, flags: MemFlags, host_ptr: Option<NonNull<T>>) -> Result<Self> {
        match rows.checked_mul(cols) {
            Some(0) | None => Err(Error::new(ErrorType::InvalidBufferSize, "overflow multiplying 'rows' and 'cols'")),
            Some(len) => {
                let inner = Buffer::create_in(ctx, len, flags, host_ptr)?;
                let rows = NonZeroUsize::new_unchecked(rows);
                let cols = NonZeroUsize::new_unchecked(cols);
                Ok(Self { inner, rows, cols, })
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
        BufferRect2D::<U, C> { inner: self.inner.transmute(), rows: self.rows, cols: self.cols }
    }
}

impl<T: Copy, C: Context> BufferRect2D<T, C> {
    #[inline(always)]
    pub fn rows (&self) -> NonZeroUsize {
        self.rows
    }

    #[inline(always)]
    pub fn cols (&self) -> NonZeroUsize {
        self.cols
    }

    #[inline(always)]
    pub fn row_pitch (&self) -> Option<usize> {
        self.rows.get().checked_mul(core::mem::size_of::<T>())
    }

    #[inline(always)]
    pub fn slice_pitch (&self) -> Option<usize> {
        self.row_pitch().and_then(|x| x.checked_mul(self.cols.get()))
    }
}

impl<T: Copy + Unpin, C: Context> BufferRect2D<T, C> {
    #[inline(always)]
    pub fn read<'src> (&'src self, slice: impl IntoSlice2D, wait: impl Into<WaitList>) -> Result<ReadBufferRect2D<'src, T>> {
        unsafe { ReadBufferRect2D::new(self, self.rows().get(), self.cols().get(), slice, Some(self.row_pitch().unwrap()), Some(self.slice_pitch().unwrap()), self.inner.ctx.next_queue(), wait) }
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