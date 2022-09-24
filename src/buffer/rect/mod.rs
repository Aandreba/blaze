flat_mod!(host);

use std::{ptr::NonNull, ops::{Deref, DerefMut}, num::NonZeroUsize, mem::MaybeUninit, fmt::Debug};
use crate::{prelude::*};
use super::{Buffer, flags::{MemFlags, MemAccess, HostPtr}};

/// Buffer that conatins a 2D rectangle.
pub struct BufferRect2D<T, C: Context = Global> {
    inner: Buffer<T, C>,
    width: NonZeroUsize,
    height: NonZeroUsize
}

impl<T> BufferRect2D<T> {
    /// Creates a new rectangular buffer from the specified values in [row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).
    #[inline(always)]
    pub fn new (v: &[T], width: usize, access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
        Self::new_in(Global, v, width, access, alloc)
    }

    #[inline(always)]
    pub fn from_rect (v: &Rect2D<T>, access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
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

impl<T, C: Context> BufferRect2D<T, C> {
    /// Creates a new rectangular buffer, in the specified context, from the specified values in [row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order).
    #[inline]
    pub fn new_in (ctx: C, v: &[T], width: usize, access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
        let height = v.len() / width;
        let host = MemFlags::new(access, HostPtr::new(alloc, true));
        unsafe { Self::create_in(ctx, width, height, host, NonNull::new(v.as_ptr() as *mut _)) }
    }

    /// Creates new rectangular buffer
    #[inline]
    pub fn from_rect_in (ctx: C, v: &Rect2D<T>, access: MemAccess, alloc: bool) -> Result<Self> where T: Copy {
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

impl<T, C: Context> Deref for BufferRect2D<T, C> {
    type Target = Buffer<T, C>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, C: Context> DerefMut for BufferRect2D<T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Unpin + PartialEq, C: Context + Clone> PartialEq for BufferRect2D<T, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && 
        self.height == other.height &&
        self.inner == other.inner
    }
}

impl<T: Unpin + Debug, C: Context + Clone> Debug for BufferRect2D<T, C> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map = Buffer::map_blocking(&self, .., None).map_err(|_| std::fmt::Error)?;
        f.debug_list().entries(map.chunks(self.width.get())).finish()
    }
}

impl<T: Unpin + Eq, C: Context + Clone> Eq for BufferRect2D<T, C> {}