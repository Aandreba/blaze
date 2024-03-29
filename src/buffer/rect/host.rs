use blaze_proc::docfg;
use std::{
    alloc::Layout,
    mem::MaybeUninit,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
    ptr::NonNull,
};
use utils_atomics::AllocError;

pub type RectBox2D<T> = Box<Rect2D<T>>;
#[docfg(feature = "svm")]
pub type SvmRect2D<T, C = crate::prelude::Global> = Box<Rect2D<T>, crate::svm::Svm<C>>;

/// A 2D rectangle stored in host memory in [row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order)
pub struct Rect2D<T> {
    width: NonZeroUsize,
    inner: [T],
}

impl<T> Rect2D<T> {
    #[inline(always)]
    pub fn new(v: &[T], width: usize) -> Box<Self>
    where
        T: Copy,
    {
        Self::try_new(v, width).unwrap()
    }

    #[inline(always)]
    pub fn new_uninit(width: usize, height: usize) -> RectBox2D<MaybeUninit<T>> {
        Self::try_new_uninit(width, height).unwrap()
    }

    #[docfg(feature = "svm")]
    #[inline(always)]
    pub fn new_in<A: std::alloc::Allocator>(v: &[T], width: usize, alloc: A) -> Box<Self, A>
    where
        T: Copy,
    {
        Self::try_new_in(v, width, alloc).unwrap()
    }

    #[docfg(feature = "svm")]
    #[inline(always)]
    pub fn new_uninit_in<A: std::alloc::Allocator>(
        width: usize,
        height: usize,
        alloc: A,
    ) -> Box<Rect2D<MaybeUninit<T>>, A> {
        Self::try_new_uninit_in(width, height, alloc).unwrap()
    }

    #[inline(always)]
    pub fn try_new(v: &[T], width: usize) -> Result<Box<Self>, AllocError>
    where
        T: Copy,
    {
        let width = NonZeroUsize::new(width).ok_or(AllocError)?;
        let (layout, delta) = Self::calculate_layout(v.len())?;

        let ptr = match layout.size() {
            0 => NonNull::dangling(),
            _ => unsafe { NonNull::new(std::alloc::alloc(layout)).ok_or(AllocError)? },
        }
        .as_ptr();

        unsafe {
            (ptr as *mut NonZeroUsize).write(width);
            core::ptr::copy_nonoverlapping(
                v.as_ptr(),
                ptr.cast::<u8>().add(delta).cast::<T>(),
                v.len(),
            );

            let raw =
                core::ptr::slice_from_raw_parts_mut::<T>(ptr as *mut T, v.len()) as *mut Rect2D<T>;
            return Ok(Box::from_raw(raw));
        }
    }

    #[inline(always)]
    pub fn try_new_uninit(
        width: usize,
        height: usize,
    ) -> Result<RectBox2D<MaybeUninit<T>>, AllocError> {
        let len = width.checked_mul(height).ok_or(AllocError)?;
        let (layout, _) = Self::calculate_layout(len)?;

        let ptr = match layout.size() {
            0 => NonNull::dangling(),
            _ => unsafe { NonNull::new(std::alloc::alloc(layout)).ok_or(AllocError)? },
        };

        let raw = core::ptr::slice_from_raw_parts_mut::<MaybeUninit<T>>(
            ptr.as_ptr() as *mut MaybeUninit<T>,
            len,
        ) as *mut Rect2D<MaybeUninit<T>>;
        return unsafe { Ok(Box::from_raw(raw)) };
    }

    #[docfg(feature = "svm")]
    pub fn try_new_in<A: std::alloc::Allocator>(
        v: &[T],
        width: usize,
        alloc: A,
    ) -> Result<Box<Self, A>, AllocError>
    where
        T: Copy,
    {
        let width = NonZeroUsize::new(width).ok_or(AllocError)?;
        let (layout, delta) = Self::calculate_layout(v.len())?;
        let ptr = alloc.allocate(layout)?.as_ptr();

        unsafe {
            (ptr as *mut NonZeroUsize).write(width);
            core::ptr::copy_nonoverlapping(
                v.as_ptr(),
                ptr.cast::<u8>().add(delta).cast::<T>(),
                v.len(),
            );

            let raw =
                core::ptr::slice_from_raw_parts_mut::<T>(ptr as *mut T, v.len()) as *mut Rect2D<T>;
            return Ok(Box::from_raw_in(raw, alloc));
        }
    }

    #[docfg(feature = "svm")]
    pub fn try_new_uninit_in<A: std::alloc::Allocator>(
        width: usize,
        height: usize,
        alloc: A,
    ) -> Result<Box<Rect2D<MaybeUninit<T>>, A>, AllocError> {
        let len = width.checked_mul(height).ok_or(AllocError)?;
        let (layout, _) = Self::calculate_layout(len)?;

        let ptr = alloc.allocate(layout)?;
        let raw = core::ptr::slice_from_raw_parts_mut::<MaybeUninit<T>>(
            ptr.as_ptr() as *mut MaybeUninit<T>,
            len,
        ) as *mut Rect2D<MaybeUninit<T>>;
        return unsafe { Ok(Box::from_raw_in(raw, alloc)) };
    }

    #[inline]
    fn calculate_layout(len: usize) -> Result<(Layout, usize), AllocError> {
        let result = Layout::new::<NonZeroUsize>()
            .extend(Layout::array::<T>(len).map_err(|_| AllocError)?)
            .map_err(|_| AllocError)?;

        return Ok(result);
    }
}

impl<T> Rect2D<MaybeUninit<T>> {
    #[cfg(feature = "svm")]
    #[inline(always)]
    pub unsafe fn assume_init<A: std::alloc::Allocator>(self: Box<Self, A>) -> Box<Rect2D<T>, A> {
        let (ptr, alloc) = Box::into_raw_with_allocator(self);
        return Box::from_raw_in(ptr as *mut Rect2D<T>, alloc);
    }

    #[cfg(not(feature = "svm"))]
    #[inline(always)]
    pub unsafe fn assume_init(self: Box<Self>) -> RectBox2D<T> {
        let ptr = Box::into_raw(self);
        return Box::from_raw(ptr as *mut Rect2D<T>);
    }
}

impl<T> Rect2D<T> {
    #[inline(always)]
    pub const fn width(&self) -> usize {
        self.width.get()
    }

    #[inline(always)]
    pub const fn height(&self) -> usize {
        #[cfg(debug_assertions)]
        if self.len() % self.width() != 0 {
            panic!("non-exact size")
        }
        self.len() / self.width()
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline(always)]
    pub const fn as_parts(&self) -> (&[T], usize) {
        (&self.inner, self.width.get())
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&[T]> {
        if index >= self.height() {
            return None;
        }
        return unsafe { Some(self.get_unchecked(index)) };
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut [T]> {
        if index >= self.height() {
            return None;
        }
        return unsafe { Some(self.get_mut_unchecked(index)) };
    }

    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> &[T] {
        core::slice::from_raw_parts(self.inner.as_ptr().add(index * self.width()), self.width())
    }

    #[inline(always)]
    pub unsafe fn get_mut_unchecked(&mut self, index: usize) -> &mut [T] {
        core::slice::from_raw_parts_mut(
            self.inner.as_mut_ptr().add(index * self.width()),
            self.width(),
        )
    }

    #[inline(always)]
    pub const fn as_slice(&self) -> &[T] {
        &self.inner
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.inner
    }

    #[inline(always)]
    pub const fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
    }
}

impl<T> Index<usize> for Rect2D<T> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of range")
    }
}

impl<T> IndexMut<usize> for Rect2D<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of range")
    }
}

/*
impl<T> RectBox2D<T> {
    #[inline(always)]
    pub fn new (v: &[T], width: usize) -> Option<Self> where T: Copy {
        Self::new_in(v, width, Global)
    }

    #[inline(always)]
    pub fn new_col_major (v: &[T], height: usize) -> Option<Self> where T: Copy {
        Self::new_col_major_in(v, height, Global)
    }

    #[inline(always)]
    pub fn new_uninit (width: usize, height: usize) -> Option<RectBox2D<MaybeUninit<T>>> {
        Self::new_uninit_in(width, height, Global)
    }

    #[inline(always)]
    pub fn new_zeroed (width: usize, height: usize) -> Option<RectBox2D<MaybeUninit<T>>> {
        Self::new_zeroed_in(width, height, Global)
    }

    #[inline(always)]
    pub const unsafe fn from_raw_parts (ptr: NonNull<T>, width: NonZeroUsize, height: NonZeroUsize) -> Self {
        Self::from_raw_parts_with_allocator(ptr, width, height, std::alloc::Global)
    }

    #[inline(always)]
    pub unsafe fn into_raw_parts (self) -> (NonNull<T>, NonZeroUsize, NonZeroUsize) {
        let this = ManuallyDrop::new(self);
        (this.ptr, this.width, this.height)
    }
}

impl<T, A: Allocator> RectBox2D<T, A> {
    #[inline(always)]
    pub fn as_ptr (&self) -> *const T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr (&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub fn allocator (&self) -> &A {
        &self.alloc
    }

    #[inline(always)]
    pub fn width (&self) -> usize {
        self.width.get()
    }

    #[inline(always)]
    pub fn height (&self) -> usize {
        self.height.get()
    }

    #[inline(always)]
    pub fn len (&self) -> usize {
        self.width() * self.height()
    }

    #[inline(always)]
    pub fn rows_iter (&self) -> Rows<'_, T, A> {
        Rows {
            inner: self,
            idx: 0,
        }
    }

    #[inline(always)]
    pub fn rows_iter_mut (&mut self) -> RowsMut<'_, T, A> {
        RowsMut {
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
    pub fn as_slice (&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.width() * self.height()) }
    }

    #[inline(always)]
    pub fn as_mut_slice (&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.width() * self.height()) }
    }

    #[inline(always)]
    pub unsafe fn transmute<U> (self) -> RectBox2D<U, A> {
        let this = ManuallyDrop::new(self);
        let alloc = core::ptr::read(addr_of!(this.alloc));
        RectBox2D { ptr: this.ptr.cast(), width: this.width, height: this.height, alloc }
    }

    #[inline(always)]
    pub fn transpose (&self) -> Self where T: Copy, A: Clone {
        let len = self.width() * self.height();
        let layout = Layout::array::<MaybeUninit<T>>(len).unwrap();
        let ptr = self.alloc.allocate(layout).unwrap().cast::<T>();

        unsafe {
            let mut result = Self::from_raw_parts_with_allocator(ptr, self.height, self.width, self.alloc.clone());

            for i in 0..result.width() {
                for j in 0..result.height() {
                    *result.get_unchecked_mut(i, j) = self.get_unchecked_copy(j, i)
                }
            }

            result
        }
    }

    #[inline(always)]
    pub fn transpose_clone (&self) -> Self where T: Clone, A: Clone {
        let len = self.width() * self.height();
        let layout = Layout::array::<MaybeUninit<T>>(len).unwrap();
        let ptr = self.alloc.allocate(layout).unwrap().cast::<T>();

        unsafe {
            let mut result = Self::from_raw_parts_with_allocator(ptr, self.height, self.width, self.alloc.clone());

            for i in 0..result.width() {
                for j in 0..result.height() {
                    *result.get_unchecked_mut(i, j) = self.get_unchecked(j, i).clone()
                }
            }

            result
        }
    }

    #[inline]
    pub fn get (&self, row: usize, col: usize) -> Option<&T> {
        if row >= self.height() || col >= self.width() {
            return None
        }

        unsafe { Some(self.get_unchecked(row, col)) }
    }

    #[inline]
    pub fn get_mut (&mut self, row: usize, col: usize) -> Option<&mut T> {
        if row >= self.height() || col >= self.width() {
            return None
        }

        unsafe { Some(self.get_unchecked_mut(row, col)) }
    }

    #[inline]
    pub fn get_copy (&self, row: usize, col: usize) -> Option<T> where T: Copy {
        if row >= self.height() || col >= self.width() {
            return None
        }

        unsafe { Some(self.get_unchecked_copy(row, col)) }
    }

    #[inline]
    pub fn get_row (&self, idx: usize) -> Option<&[T]> {
        if idx >= self.height() {
            return None
        }

        unsafe { Some(self.get_row_unckecked(idx)) }
    }

    #[inline]
    pub fn get_row_mut (&mut self, idx: usize) -> Option<&mut [T]> {
        if idx >= self.height() {
            return None
        }

        unsafe { Some(self.get_row_unckecked_mut(idx)) }
    }

    #[inline]
    pub unsafe fn get_unchecked (&self, x: usize, y: usize) -> &T {
        let offset = y * self.width() + x;
        &*self.ptr.as_ptr().add(offset)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut (&mut self, x: usize, y: usize) -> &mut T {
        let offset = y * self.width() + x;
        &mut *self.ptr.as_ptr().add(offset)
    }

    #[inline]
    pub unsafe fn get_unchecked_copy (&self, x: usize, y: usize) -> T where T: Copy {
        let offset = y * self.width() + x;
        *self.ptr.as_ptr().add(offset)
    }

    #[inline]
    pub unsafe fn get_row_unckecked (&self, y: usize) -> &[T] {
        let ptr = self.ptr.as_ptr().add(y * self.width());
        core::slice::from_raw_parts(ptr, self.width())
    }

    #[inline]
    pub unsafe fn get_row_unckecked_mut (&mut self, y: usize) -> &mut [T] {
        let ptr = self.ptr.as_ptr().add(y * self.width());
        core::slice::from_raw_parts_mut(ptr, self.width())
    }

    pub fn new_in (v: &[T], width: usize, alloc: A) -> Option<Self> where T: Copy {
        let width = NonZeroUsize::new(width)?;
        let height = NonZeroUsize::new(v.len() / width)?;

        let layout = Layout::array::<T>(v.len()).ok()?;
        let ptr = alloc.allocate(layout).ok()?.cast();

        unsafe {
            core::ptr::copy_nonoverlapping(v.as_ptr(), ptr.as_ptr(), v.len())
        }

        Some(Self { ptr, alloc, width, height })
    }

    pub fn new_col_major_in (v: &[T], height: usize, alloc: A) -> Option<Self> where T: Copy {
        let height = NonZeroUsize::new(height)?;
        let width = NonZeroUsize::new(v.len() / height)?;

        let layout = Layout::array::<MaybeUninit<T>>(v.len()).ok()?;
        let ptr = alloc.allocate(layout).unwrap().cast::<T>();

        unsafe {
            let mut result = Self::from_raw_parts_with_allocator(ptr, width, height, alloc);

            for i in 0..result.width() {
                for j in 0..result.height() {
                    let idx_v = i * height.get() + j;
                    *result.get_unchecked_mut(i, j) = v[idx_v];
                }
            }

            Some(result)
        }
    }

    pub fn new_uninit_in (width: usize, height: usize, alloc: A) -> Option<RectBox2D<MaybeUninit<T>, A>> {
        let len = match width.checked_mul(height) {
            Some(0) | None => return None,
            Some(len) => len
        };

        let layout = Layout::array::<MaybeUninit<T>>(len).ok()?;

        unsafe {
            let ptr = alloc.allocate(layout).ok()?.cast();
            let width = NonZeroUsize::new_unchecked(width);
            let height = NonZeroUsize::new_unchecked(height);

            Some(RectBox2D {
                ptr,
                alloc,
                width,
                height
            })
        }
    }

    pub fn new_zeroed_in (width: usize, height: usize, alloc: A) -> Option<RectBox2D<MaybeUninit<T>, A>> {
        let len = match width.checked_mul(height) {
            Some(0) | None => return None,
            Some(len) => len
        };

        let layout = Layout::array::<MaybeUninit<T>>(len).ok()?;

        unsafe {
            let ptr = alloc.allocate_zeroed(layout).ok()?.cast();
            let width = NonZeroUsize::new_unchecked(width);
            let height = NonZeroUsize::new_unchecked(height);

            Some(RectBox2D {
                ptr,
                alloc,
                width,
                height
            })
        }
    }

    #[inline(always)]
    pub const unsafe fn from_raw_parts_with_allocator (ptr: NonNull<T>, width: NonZeroUsize, height: NonZeroUsize, alloc: A) -> Self {
        Self { ptr, alloc, width, height }
    }

    #[inline(always)]
    pub unsafe fn into_raw_parts_with_allocator (self) -> (NonNull<T>, NonZeroUsize, NonZeroUsize, A) {
        let this = ManuallyDrop::new(self);
        let alloc = core::ptr::read(addr_of!(this.alloc));
        (this.ptr, this.width, this.height, alloc)
    }

    #[inline]
    pub fn from_boxed_slice (v: Box<[T], A>, width: usize) -> Option<Self> {
        let width = NonZeroUsize::new(width)?;
        let height = NonZeroUsize::new(v.len() / width)?;

        let (ptr, alloc) = Box::into_raw_with_allocator(v);
        let ptr = NonNull::new(ptr as *mut T)?;

        unsafe { Some(Self::from_raw_parts_with_allocator(ptr, width, height, alloc)) }
    }

    #[inline]
    pub fn into_boxed_slice (self) -> Box<[T], A> {
        let (ptr, width, height, alloc) = unsafe { self.into_raw_parts_with_allocator() };
        let len = width.checked_mul(height).unwrap();

        unsafe {
            let slice = core::slice::from_raw_parts_mut(ptr.as_ptr(), len.get());
            Box::from_raw_in(slice, alloc)
        }
    }

    #[inline(always)]
    pub fn into_vec (self) -> Vec<T, A> {
        self.into_boxed_slice().into_vec()
    }
}

impl<T, A: Allocator> RectBox2D<MaybeUninit<T>, A> {
    #[inline(always)]
    pub unsafe fn assume_init (self) -> RectBox2D<T, A> {
        assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<MaybeUninit<T>>());
        let this = ManuallyDrop::new(self);
        let alloc = core::ptr::read(addr_of!(this.alloc));

        RectBox2D {
            ptr: this.ptr.cast(),
            width: this.width,
            height: this.height,
            alloc
        }
    }
}

impl<T: PartialEq, A: Allocator> PartialEq for RectBox2D<T, A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width &&
        self.height == other.height &&
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, A: Allocator> Eq for RectBox2D<T, A> {}

#[docfg(feature = "svm")]
unsafe impl<T, C: crate::prelude::Context> crate::svm::SvmPointer<T> for SvmRect2D<T, C> {
    type Context = C;

    #[inline(always)]
    fn allocator (&self) -> &crate::svm::Svm<C> {
        &self.alloc
    }

    #[inline(always)]
    fn as_ptr (&self) -> *const T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    fn as_mut_ptr (&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    fn len (&self) -> usize {
        self.width() * self.height()
    }
}

#[docfg(feature = "svm")]
unsafe impl<T: Sync, C: crate::prelude::Context> crate::buffer::KernelPointer<T> for SvmRect2D<T, C> where C: 'static + Send + Clone {
    #[inline]
    unsafe fn set_arg (&self, kernel: &mut crate::prelude::RawKernel, wait: &mut Vec<crate::prelude::RawEvent>, idx: u32) -> crate::prelude::Result<()> {
        kernel.set_svm_argument::<T, Self>(idx, self)?;

        if SvmRect2D::allocator(self).is_coarse() {
            let evt = SvmRect2D::allocator(self).unmap(crate::svm::SvmPointer::<T>::as_ptr(self) as *mut _, None)?;
            wait.push(evt)
        }

        Ok(())
    }

    #[inline]
    fn complete (&self, event: &crate::prelude::RawEvent) -> crate::prelude::Result<()> {
        if SvmRect2D::allocator(self).is_coarse() {
            let alloc = SvmRect2D::allocator(self);
            let size = core::mem::size_of::<T>() * crate::svm::SvmPointer::<T>::len(self);

            unsafe {
                let _ = alloc.map::<{opencl_sys::CL_MAP_READ | opencl_sys::CL_MAP_WRITE}>(
                    self.as_ptr() as *mut _,
                    size,
                    Some(core::slice::from_ref(event))
                )?;
            }
        }

        Ok(())
    }
}

impl<T, A: Allocator> Index<usize> for RectBox2D<T, A> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.height.get());
        unsafe { self.get_row_unckecked(index) }
    }
}

impl<T, A: Allocator> Index<(usize, usize)> for RectBox2D<T, A> {
    type Output = T;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        assert!(x < self.width.get());
        assert!(y < self.height.get());
        unsafe { self.get_unchecked(x, y) }
    }
}

impl<T, A: Allocator> Index<[usize; 2]> for RectBox2D<T, A> {
    type Output = T;

    #[inline]
    fn index(&self, index: [usize; 2]) -> &Self::Output {
        let [x, y] = index;
        assert!(x < self.width.get());
        assert!(y < self.height.get());
        unsafe { self.get_unchecked(x, y) }
    }
}

impl<T, A: Allocator> IndexMut<usize> for RectBox2D<T, A> {
    #[inline]
    fn index_mut (&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.height.get());
        unsafe { self.get_row_unckecked_mut(index) }
    }
}

impl<T, A: Allocator> IndexMut<(usize, usize)> for RectBox2D<T, A> {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        assert!(x < self.width.get());
        assert!(y < self.height.get());
        unsafe { self.get_unchecked_mut(x, y) }
    }
}

impl<T, A: Allocator> IndexMut<[usize; 2]> for RectBox2D<T, A> {
    #[inline]
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        let [x, y] = index;
        assert!(x < self.width.get());
        assert!(y < self.height.get());
        unsafe { self.get_unchecked_mut(x, y) }
    }
}

impl<T, A: Allocator> Drop for RectBox2D<T, A> {
    #[inline]
    fn drop(&mut self) {
        let len = self.width.get() * self.height.get();
        let layout = Layout::array::<T>(len).unwrap();

        unsafe {
            for i in 0..len {
                self.ptr.as_ptr().add(i).drop_in_place()
            }

            self.alloc.deallocate(self.ptr.cast(), layout)
        }
    }
}

impl<T: Debug, A: Allocator> Debug for RectBox2D<T, A> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.rows_iter()).finish()
    }
}

unsafe impl<T: Send, A: Send + Allocator> Send for RectBox2D<T, A> {}
unsafe impl<T: Sync, A: Sync + Allocator> Sync for RectBox2D<T, A> {}

pub struct Rows<'a, T, A: Allocator = Global> {
    pub(super) inner: &'a RectBox2D<T, A>,
    pub(super) idx: usize
}

impl<'a, T, A: Allocator> Iterator for Rows<'a, T, A> {
    type Item = &'a [T];

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.inner.get_row(self.idx) {
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

impl<'a, T, A: Allocator> ExactSizeIterator for Rows<'a, T, A> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.height() - self.idx
    }
}

pub struct RowsMut<'a, T, A: Allocator = Global> {
    pub(super) inner: &'a mut RectBox2D<T, A>,
    pub(super) idx: usize
}

impl<'a, T, A: Allocator> Iterator for RowsMut<'a, T, A> {
    type Item = &'a mut [T];

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.inner.height() {
            unsafe {
                let ptr = self.inner.ptr.as_ptr().add(self.idx * self.inner.width());
                let v = core::slice::from_raw_parts_mut(ptr, self.inner.width());

                self.idx += 1;
                return Some(v)
            }
        }

        None
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T, A: Allocator> ExactSizeIterator for RowsMut<'a, T, A> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.height() - self.idx
    }
}

pub struct Cols<'a, T, A: Allocator = Global> {
    pub(super) inner: &'a RectBox2D<T, A>,
    pub(super) idx: usize
}

impl<'a, T, A: Allocator> Iterator for Cols<'a, T, A> {
    type Item = Col<'a, T, A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.inner.width() {
            return None
        }

        let iter = Col {
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

impl<'a, T, A: Allocator> ExactSizeIterator for Cols<'a, T, A> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.width() - self.idx
    }
}

pub struct Col<'a, T, A: Allocator = Global> {
    inner: &'a RectBox2D<T, A>,
    row: usize,
    idx: usize
}

impl<'a, T, A: Allocator> Clone for Col<'a, T, A> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), row: self.row.clone(), idx: self.idx.clone() }
    }
}

impl<'a, T, A: Allocator> Index<usize> for Col<'a, T, A> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[(self.row, self.idx + index)]
    }
}

impl<'a, T, A: Allocator> Iterator for Col<'a, T, A> {
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

impl<'a, T, A: Allocator> ExactSizeIterator for Col<'a, T, A> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.height() - self.idx
    }
}

impl<'a, T: Debug, A: Allocator> Debug for Col<'a, T, A> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

impl<'a, T, A: Allocator> Copy for Col<'a, T, A> {}*/
