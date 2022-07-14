use std::{ptr::{NonNull, addr_of}, num::NonZeroUsize, mem::{MaybeUninit, ManuallyDrop}, alloc::{Allocator, Global, Layout}, ops::{Index, IndexMut}, fmt::Debug};

/// A 2D rectangle stored in host memory in [column-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order)
pub struct Rect2D<T, A: Allocator = Global> {
    ptr: NonNull<T>,
    alloc: A,
    rows: NonZeroUsize,
    cols: NonZeroUsize
}

impl<T> Rect2D<T> {
    #[inline(always)]
    pub fn new (v: &[T], rows: usize) -> Option<Self> where T: Copy {
        Self::new_in(v, rows, Global)
    }

    #[inline(always)]
    pub fn new_row_major (v: &[T], rows: usize) -> Option<Self> where T: Copy {
        Self::new_row_major_in(v, rows, Global)
    }

    #[inline(always)]
    pub fn new_uninit (rows: usize, cols: usize) -> Option<Rect2D<MaybeUninit<T>>> {
        Self::new_uninit_in(rows, cols, Global)
    }

    #[inline(always)]
    pub fn new_zeroed (rows: usize, cols: usize) -> Option<Rect2D<MaybeUninit<T>>> {
        Self::new_zeroed_in(rows, cols, Global)
    }
}

impl<T, A: Allocator> Rect2D<T, A> {
    #[inline(always)]
    pub fn as_ptr (&self) -> *const T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr (&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub fn rows (&self) -> usize {
        self.rows.get()
    }

    #[inline(always)]
    pub fn cols (&self) -> usize {
        self.cols.get()
    }

    #[inline(always)]
    pub fn rows_iter (&self) -> Rows<'_, T, A> {
        Rows {
            inner: self,
            idx: 0,
        }
    }

    #[inline(always)]
    pub fn cols_iter (&self) -> Cols<'_, T, A> {
        Cols {
            inner: self,
            idx: 0,
        }
    }

    #[inline(always)]
    pub fn transpose (&self) -> Self where T: Copy, A: Clone {
        let len = self.rows() * self.cols();
        let layout = Layout::array::<MaybeUninit<T>>(len).unwrap();
        let ptr = self.alloc.allocate(layout).unwrap().cast::<T>();

        unsafe {
            let mut result = Self::from_raw_parts_with_allocator(ptr, self.cols, self.rows, self.alloc.clone());

            for i in 0..result.rows() {
                for j in 0..result.cols() {
                    *result.get_unchecked_mut(i, j) = self.get_unchecked_copy(j, i)
                }
            }

            result
        }
    }

    #[inline(always)]
    pub fn transpose_clone (&self) -> Self where T: Clone, A: Clone {
        let len = self.rows() * self.cols();
        let layout = Layout::array::<MaybeUninit<T>>(len).unwrap();
        let ptr = self.alloc.allocate(layout).unwrap().cast::<T>();

        unsafe {
            let mut result = Self::from_raw_parts_with_allocator(ptr, self.cols, self.rows, self.alloc.clone());

            for i in 0..result.rows() {
                for j in 0..result.cols() {
                    *result.get_unchecked_mut(i, j) = self.get_unchecked(j, i).clone()
                }
            }

            result
        }
    }

    #[inline]
    pub fn get (&self, row: usize, col: usize) -> Option<&T> {
        if row >= self.rows.get() || col >= self.cols.get() {
            return None
        }

        unsafe { Some(self.get_unchecked(row, col)) }
    }

    #[inline]
    pub fn get_mut (&mut self, row: usize, col: usize) -> Option<&mut T> {
        if row >= self.rows.get() || col >= self.cols.get() {
            return None
        }

        unsafe { Some(self.get_unchecked_mut(row, col)) }
    }

    #[inline]
    pub fn get_copy (&self, row: usize, col: usize) -> Option<T> where T: Copy {
        if row >= self.rows.get() || col >= self.cols.get() {
            return None
        }

        unsafe { Some(self.get_unchecked_copy(row, col)) }
    }

    #[inline]
    pub fn get_col (&self, idx: usize) -> Option<&[T]> {
        if idx >= self.cols.get() {
            return None
        }

        unsafe { Some(self.get_col_unckecked(idx)) }
    }

    #[inline]
    pub fn get_col_mut (&mut self, idx: usize) -> Option<&mut [T]> {
        if idx >= self.cols.get() {
            return None
        }

        unsafe { Some(self.get_col_unckecked_mut(idx)) }
    }

    #[inline]
    pub unsafe fn get_unchecked (&self, row: usize, col: usize) -> &T {
        let offset = col * self.rows.get() + row;
        &*self.ptr.as_ptr().add(offset)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut (&mut self, row: usize, col: usize) -> &mut T {
        let offset = col * self.rows.get() + row;
        &mut *self.ptr.as_ptr().add(offset)
    }

    #[inline]
    pub unsafe fn get_unchecked_copy (&self, row: usize, col: usize) -> T where T: Copy {
        let offset = col * self.rows.get() + row;
        *self.ptr.as_ptr().add(offset)
    }

    #[inline]
    pub unsafe fn get_col_unckecked (&self, idx: usize) -> &[T] {
        let ptr = self.ptr.as_ptr().add(idx * self.rows());
        core::slice::from_raw_parts(ptr, self.rows())
    }

    #[inline]
    pub unsafe fn get_col_unckecked_mut (&mut self, idx: usize) -> &mut [T] {
        let ptr = self.ptr.as_ptr().add(idx * self.rows());
        core::slice::from_raw_parts_mut(ptr, self.rows())
    }

    #[inline(always)]
    pub const unsafe fn from_raw_parts_with_allocator (ptr: NonNull<T>, rows: NonZeroUsize, cols: NonZeroUsize, alloc: A) -> Self {
        Self { ptr, alloc, rows, cols }
    }

    pub fn new_in (v: &[T], rows: usize, alloc: A) -> Option<Self> where T: Copy {
        let rows = NonZeroUsize::new(rows)?;
        let cols = NonZeroUsize::new(v.len() / rows)?;

        let layout = Layout::array::<T>(v.len()).ok()?;
        let ptr = alloc.allocate(layout).ok()?.cast();

        unsafe {
            core::ptr::copy_nonoverlapping(v.as_ptr(), ptr.as_ptr(), v.len())
        }

        Some(Self { ptr, alloc, rows, cols })
    }

    pub fn new_row_major_in (v: &[T], rows: usize, alloc: A) -> Option<Self> where T: Copy {
        let rows = NonZeroUsize::new(rows)?;
        let cols = NonZeroUsize::new(v.len() / rows)?;

        let layout = Layout::array::<MaybeUninit<T>>(v.len()).ok()?;
        let ptr = alloc.allocate(layout).unwrap().cast::<T>();

        unsafe {
            let mut result = Self::from_raw_parts_with_allocator(ptr, rows, cols, alloc);

            for i in 0..result.rows() {
                for j in 0..result.cols() {
                    let idx_v = i * cols.get() + j;
                    *result.get_unchecked_mut(i, j) = v[idx_v];
                }
            }

            Some(result)
        }
    }

    pub fn new_uninit_in (rows: usize, cols: usize, alloc: A) -> Option<Rect2D<MaybeUninit<T>, A>> {
        let len = match rows.checked_mul(cols) {
            Some(0) | None => return None,
            Some(len) => len
        };

        let layout = Layout::array::<MaybeUninit<T>>(len).ok()?;

        unsafe {
            let ptr = alloc.allocate(layout).ok()?.cast();
            let width = NonZeroUsize::new_unchecked(rows);
            let height = NonZeroUsize::new_unchecked(cols);
            
            Some(Rect2D {
                ptr,
                alloc,
                rows: width,
                cols: height
            })
        }
    }

    pub fn new_zeroed_in (rows: usize, cols: usize, alloc: A) -> Option<Rect2D<MaybeUninit<T>, A>> {
        let len = match rows.checked_mul(cols) {
            Some(0) | None => return None,
            Some(len) => len
        };

        let layout = Layout::array::<MaybeUninit<T>>(len).ok()?;

        unsafe {
            let ptr = alloc.allocate_zeroed(layout).ok()?.cast();
            let width = NonZeroUsize::new_unchecked(rows);
            let height = NonZeroUsize::new_unchecked(cols);
            
            Some(Rect2D {
                ptr,
                alloc,
                rows: width,
                cols: height
            })
        }
    }
}

impl<T, A: Allocator> Rect2D<MaybeUninit<T>, A> {
    #[inline(always)]
    pub unsafe fn assume_init (self) -> Rect2D<T, A> {
        assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<MaybeUninit<T>>());

        let this = ManuallyDrop::new(self);
        let alloc = core::ptr::read(addr_of!(this.alloc));

        Rect2D { 
            ptr: this.ptr.cast(),
            rows: this.rows,
            cols: this.cols,
            alloc
        }
    }
}

impl<T, A: Allocator> Index<usize> for Rect2D<T, A> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.cols.get());
        unsafe { self.get_col_unckecked(index) }
    }
}

impl<T, A: Allocator> Index<(usize, usize)> for Rect2D<T, A> {
    type Output = T;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (row, col) = index;
        assert!(row < self.rows.get());
        assert!(col < self.cols.get());
        unsafe { self.get_unchecked(row, col) }
    }
}

impl<T, A: Allocator> Index<[usize; 2]> for Rect2D<T, A> {
    type Output = T;

    #[inline]
    fn index(&self, index: [usize; 2]) -> &Self::Output {
        let [row, col] = index;
        assert!(row < self.rows.get());
        assert!(col < self.cols.get());
        unsafe { self.get_unchecked(row, col) }
    }
}

impl<T, A: Allocator> IndexMut<usize> for Rect2D<T, A> {
    #[inline]
    fn index_mut (&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.cols.get());
        unsafe { self.get_col_unckecked_mut(index) }
    }
}

impl<T, A: Allocator> IndexMut<(usize, usize)> for Rect2D<T, A> {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (row, col) = index;
        assert!(row < self.rows.get());
        assert!(col < self.cols.get());
        unsafe { self.get_unchecked_mut(row, col) }
    }
}

impl<T, A: Allocator> IndexMut<[usize; 2]> for Rect2D<T, A> {
    #[inline]
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        let [row, col] = index;
        assert!(row < self.rows.get());
        assert!(col < self.cols.get());
        unsafe { self.get_unchecked_mut(row, col) }
    }
}

impl<T, A: Allocator> Drop for Rect2D<T, A> {
    #[inline]
    fn drop(&mut self) {
        let len = self.rows.get() * self.cols.get();
        let layout = Layout::array::<T>(len).unwrap();

        unsafe {
            for i in 0..len {
                self.ptr.as_ptr().add(i).drop_in_place()
            }
    
            self.alloc.deallocate(self.ptr.cast(), layout)
        }
    }
}

impl<T: Debug, A: Allocator> Debug for Rect2D<T, A> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.rows_iter()).finish()
    }
}

unsafe impl<T: Send, A: Send + Allocator> Send for Rect2D<T, A> {}
unsafe impl<T: Sync, A: Sync + Allocator> Sync for Rect2D<T, A> {}

pub struct Cols<'a, T, A: Allocator = Global> {
    pub(super) inner: &'a Rect2D<T, A>,
    pub(super) idx: usize
}

impl<'a, T, A: Allocator> Iterator for Cols<'a, T, A> {
    type Item = &'a [T];

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.inner.get_col(self.idx) {
            self.idx += 1;
            return Some(v)
        }

        None
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T, A: Allocator> ExactSizeIterator for Cols<'a, T, A> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.cols() - self.idx
    }
}

pub struct Rows<'a, T, A: Allocator = Global> {
    pub(super) inner: &'a Rect2D<T, A>,
    pub(super) idx: usize
}

impl<'a, T, A: Allocator> Iterator for Rows<'a, T, A> {
    type Item = Row<'a, T, A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.inner.rows() {
            return None
        }

        let iter = Row {
            inner: self.inner,
            row: self.idx,
            idx: 0
        };

        self.idx += 1;
        Some(iter)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T, A: Allocator> ExactSizeIterator for Rows<'a, T, A> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.rows() - self.idx
    }
}

pub struct Row<'a, T, A: Allocator = Global> {
    inner: &'a Rect2D<T, A>,
    row: usize,
    idx: usize
}

impl<'a, T, A: Allocator> Clone for Row<'a, T, A> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), row: self.row.clone(), idx: self.idx.clone() }
    }
}

impl<'a, T, A: Allocator> Index<usize> for Row<'a, T, A> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[(self.row, self.idx + index)]
    }
}

impl<'a, T, A: Allocator> Iterator for Row<'a, T, A> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.inner.get(self.row, self.idx) {
            self.idx += 1;
            return Some(v)
        }
        
        None
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T, A: Allocator> ExactSizeIterator for Row<'a, T, A> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.cols() - self.idx
    }
}

impl<'a, T: Debug, A: Allocator> Debug for Row<'a, T, A> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

impl<'a, T, A: Allocator> Copy for Row<'a, T, A> {}