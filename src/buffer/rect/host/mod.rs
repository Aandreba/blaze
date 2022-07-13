flat_mod!(rows);
use std::{ptr::{NonNull, addr_of}, num::NonZeroUsize, mem::{MaybeUninit, ManuallyDrop}, alloc::{Allocator, Global, Layout}, ops::{Index, IndexMut, Deref}, fmt::Debug};

/// A 2D rectangle stored in host memory.
pub struct Rect2D<T, A: Allocator = Global> {
    ptr: NonNull<T>,
    alloc: A,
    rows: NonZeroUsize,
    cols: NonZeroUsize
}

impl<T> Rect2D<T> {
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
    pub fn rows (&self) -> Rows<'_, T, A> {
        Rows {
            inner: self,
            idx: 0,
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
    pub fn get_row (&self, idx: usize) -> Option<&[T]> {
        if idx >= self.rows.get() {
            return None
        }

        unsafe { Some(self.get_row_unckecked(idx)) }
    }

    #[inline]
    pub fn get_row_mut (&mut self, idx: usize) -> Option<&mut [T]> {
        if idx >= self.rows.get() {
            return None
        }

        unsafe { Some(self.get_row_unckecked_mut(idx)) }
    }

    #[inline]
    pub unsafe fn get_unchecked (&self, row: usize, col: usize) -> &T {
        let offset = row * self.cols.get() + col;
        &*self.ptr.as_ptr().add(offset)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut (&mut self, row: usize, col: usize) -> &mut T {
        let offset = row * self.cols.get() + col;
        &mut *self.ptr.as_ptr().add(offset)
    }

    #[inline]
    pub unsafe fn get_unchecked_copy (&self, row: usize, col: usize) -> T where T: Copy {
        let offset = row * self.cols.get() + col;
        *self.ptr.as_ptr().add(offset)
    }

    #[inline]
    pub unsafe fn get_row_unckecked (&self, idx: usize) -> &[T] {
        let ptr = self.ptr.as_ptr().add(idx * self.cols.get());
        core::slice::from_raw_parts(ptr, self.cols.get())
    }

    #[inline]
    pub unsafe fn get_row_unckecked_mut (&mut self, idx: usize) -> &mut [T] {
        let ptr = self.ptr.as_ptr().add(idx * self.cols.get());
        core::slice::from_raw_parts_mut(ptr, self.cols.get())
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
        assert!(index < self.rows.get());
        unsafe { self.get_row_unckecked(index) }
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
        assert!(index < self.rows.get());
        unsafe { self.get_row_unckecked_mut(index) }
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
        f.debug_list().entries(self.rows()).finish()
    }
}

unsafe impl<T: Send, A: Send + Allocator> Send for Rect2D<T, A> {}
unsafe impl<T: Sync, A: Sync + Allocator> Sync for Rect2D<T, A> {}