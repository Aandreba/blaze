use std::{ptr::{NonNull, Pointee}, marker::{PhantomData, Unsize}, alloc::{Allocator, Global, Layout, AllocError}, ops::{Deref, DerefMut}, fmt::Debug, mem::ManuallyDrop, hash::Hash};

/// To be replaced with [`ThinBox`](std::boxed::ThinBox) when stabilized and feature complete
pub struct ThinBox<T: ?Sized, A: Allocator = Global> {
    inner: NonNull<<T as Pointee>::Metadata>,
    alloc: A,
    phtm: PhantomData<T>
}

impl<T, A: Allocator> ThinBox<T, A> {
    #[inline(always)]
    pub fn new_in (v: T, alloc: A) -> Self {
        Self::try_new_in(v, alloc).unwrap()
    }

    #[inline]
    pub fn try_new_in (v: T, alloc: A) -> Result<Self, AllocError> {
        let inner = alloc.allocate(Layout::new::<T>())?.cast::<()>();
        unsafe {
            inner.as_ptr().cast::<T>().write(v)
        }
        return Ok(Self { inner, alloc, phtm: PhantomData })
    }

    #[inline]
    pub fn into_inner (self) -> T {
        let this = ManuallyDrop::new(self);

        unsafe {
            let v = core::ptr::read(this.inner.as_ptr().cast::<T>());
            this.alloc.deallocate(this.inner.cast(), Layout::new::<T>());
            return v
        }
        
    }
}

impl<T: ?Sized, A: Allocator> ThinBox<T, A> {
    #[inline(always)]
    pub fn new_unsize_in<U: Unsize<T>> (v: U, alloc: A) -> Self {
        Self::try_new_unsize_in(v, alloc).unwrap()
    }
    
    pub fn try_new_unsize_in<U: Unsize<T>> (v: U, alloc: A) -> Result<Self, AllocError> {
        let unsize = &v as &T;
        let unsize_layout = Layout::for_value(unsize);

        let layout = Layout::new::<<T as Pointee>::Metadata>();
        let padding = layout.padding_needed_for(unsize_layout.align());
        let layout = Layout::from_size_align(
            layout.size() + padding + unsize_layout.size(),
            layout.align()
        ).map_err(|_| AllocError)?;

        let inner = alloc.allocate(layout)?.cast::<<T as Pointee>::Metadata>();
        unsafe {
            *inner.as_ptr() = core::ptr::metadata(unsize);
            core::ptr::copy_nonoverlapping(
                unsize as *const _ as *const u8,
                inner.as_ptr().add(1).cast::<u8>().add(padding),
                unsize_layout.size()
            );
        }

        core::mem::forget(v);
        return Ok(Self { inner, alloc, phtm: PhantomData })
    }

    #[inline(always)]
    pub fn box_size (&self) -> usize {
        let this = self.as_ref();
        let layout = Layout::new::<<T as Pointee>::Metadata>();
        let padding = layout.padding_needed_for(core::mem::align_of_val(this));
        return layout.size() + padding + core::mem::size_of_val(this);
    }

    #[inline(always)]
    pub fn box_layout (&self) -> Layout {
        return Layout::from_size_align(self.box_size(), core::mem::align_of::<<T as Pointee>::Metadata>()).unwrap()
    }

    #[inline(always)]
    pub fn as_ptr (&self) -> *const T {
        return unsafe {
            core::ptr::from_raw_parts(self.inner.as_ptr().add(1).cast(), self.metadata())
        }
    }
    
    #[inline(always)]
    pub fn as_mut_ptr (&mut self) -> *mut T {
        return unsafe {
            core::ptr::from_raw_parts_mut(self.inner.as_ptr().add(1).cast(), self.metadata())
        }
    }

    #[inline(always)]
    pub fn as_ref (&self) -> &T {
        return unsafe { &*self.as_ptr() }
    }

    #[inline(always)]
    pub fn as_mut (&mut self) -> &mut T {
        return unsafe { &mut *self.as_mut_ptr() }
    }

    #[inline(always)]
    pub fn metadata (&self) -> <T as Pointee>::Metadata {
        return unsafe { *self.inner.as_ptr() }
    }

    #[inline(always)]
    pub fn into_box (self) -> Box<T, A> {
        todo!()
    }

    #[inline(always)]
    pub fn from_box (bx: Box<T, A>) -> Self {
        let (ptr, alloc) = Box::into_raw_with_allocator(bx);

        unsafe {
            // Avoid extra allocation if metadata is zero-sized.
            if core::mem::size_of::<<T as Pointee>::Metadata>() == 0 {
                return Self { inner: NonNull::new_unchecked(ptr).cast(), alloc, phtm: PhantomData }
            }
    
            // Calculate new size
            let unsize_layout = Layout::for_value_raw(ptr);
            let layout = Layout::new::<<T as Pointee>::Metadata>();

            let padding = layout.padding_needed_for(unsize_layout.align());
            let size = layout.size() + padding + unsize_layout.size();
    
            // Allocate new pointer
            let layout = Layout::from_size_align(size, layout.align()).unwrap();
            let inner = alloc.allocate(layout).unwrap().cast::<<T as Pointee>::Metadata>();
            
            core::ptr::write(inner.as_ptr(), core::ptr::metadata(ptr));
            core::ptr::copy_nonoverlapping(
                ptr.cast::<u8>(),
                inner.as_ptr().add(1).cast::<u8>(),
                unsize_layout.size()
            );

            return Self { inner, alloc, phtm: PhantomData }
        }
    }

    #[inline(always)]
    pub fn into_raw_with_alloc (self) -> (*mut (), A) {
        unsafe {
            let v = (self.inner.as_ptr().cast(), core::ptr::read(&self.alloc));
            core::mem::forget(self);
            return v;
        }
    }

    #[inline(always)]
    pub unsafe fn from_raw_with_alloc (ptr: *mut (), alloc: A) -> Self {
        Self { inner: NonNull::new(ptr.cast()).unwrap(), alloc, phtm: PhantomData }
    }
}

impl<T: ?Sized> ThinBox<T> {
    #[inline(always)]
    pub fn new (v: T) -> Self where T: Sized {
        Self::new_in(v, Global)
    }

    #[inline(always)]
    pub fn new_unsize<U: Unsize<T>> (v: U) -> Self {
        Self::new_unsize_in(v, Global)
    }

    #[inline(always)]
    pub fn try_new (v: T) -> Result<Self, AllocError> where T: Sized {
        Self::try_new_in(v, Global)
    }

    #[inline(always)]
    pub fn try_new_unsize<U: Unsize<T>> (v: U) -> Result<Self, AllocError> {
        Self::try_new_unsize_in(v, Global)
    }

    #[inline(always)]
    pub fn into_raw (self) -> *mut () {
        let this = ManuallyDrop::new(self);
        return this.inner.as_ptr().cast();
    }

    #[inline(always)]
    pub unsafe fn from_raw (ptr: *mut ()) -> Self {
        Self::from_raw_with_alloc(ptr, Global)
    }
}

impl<T: ?Sized, A: Allocator> Deref for ThinBox<T, A> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized, A: Allocator> DerefMut for ThinBox<T, A> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: ?Sized + Debug, A: Allocator> Debug for ThinBox<T, A> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self as &T, f)
    }
}

impl<T: ?Sized + PartialEq, A: Allocator> PartialEq for ThinBox<T, A> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: ?Sized + PartialOrd, A: Allocator> PartialOrd for ThinBox<T, A> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl<T: ?Sized + Ord, A: Allocator> Ord for ThinBox<T, A> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other)
    }
}

impl<T: ?Sized + Hash, A: Allocator> Hash for ThinBox<T, A> {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T: Default, A: Allocator + Default> Default for ThinBox<T, A> {
    #[inline]
    fn default() -> Self {
        Self::new_in(Default::default(), Default::default())
    }
}

impl<T: Clone, A: Allocator + Clone> Clone for ThinBox<T, A> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new_in(self.as_ref().clone(), self.alloc.clone())
    }
}

impl<Args, F: ?Sized + FnOnce<Args>, A: Allocator> FnOnce<Args> for ThinBox<F, A> {
    type Output = F::Output;

    #[inline(always)]
    extern "rust-call" fn call_once(mut self, args: Args) -> Self::Output {
        (*self).call_once(args)
    }
}

impl<T: ?Sized, A: Allocator> Drop for ThinBox<T, A> {
    fn drop(&mut self) {
        if core::mem::needs_drop::<T>() {
            unsafe {
                core::ptr::drop_in_place(self.as_mut_ptr())
            }
        }

        unsafe {
            self.alloc.deallocate(self.inner.cast(), self.box_layout());
        }
    }
}

impl<T: ?Sized + Eq, A: Allocator> Eq for ThinBox<T, A> {}
unsafe impl<T: ?Sized + Send, A: Allocator + Send> Send for ThinBox<T, A> {}
unsafe impl<T: ?Sized + Sync, A: Allocator + Sync> Sync for ThinBox<T, A> {}

#[cfg(test)]
#[test]
fn test () {
    let thin : ThinBox<dyn Into<u32>> = ThinBox::new_unsize(3u8);
    let v: u32 = <dyn Into<u32>>::into(thin);
    println!("{:?} v. {:?}", thin.box_layout(), Layout::array::<i32>(3).unwrap());
}