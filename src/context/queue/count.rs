use std::{ptr::NonNull, sync::atomic::AtomicUsize, mem::ManuallyDrop};

/// A heap-allocated count of the number of references to a context, which also serves as the reference count of the value.
/// Compared to using an `Arc<()>`, this should save a `usize` worth of size in the heap.
#[repr(transparent)]
pub(super) struct AtomicCount {
    inner: NonNull<AtomicUsize>
}

impl AtomicCount {
    #[inline(always)]
    pub fn new () -> Self {
        let ptr = Box::into_raw(Box::new(AtomicUsize::new(1)));

        unsafe {
            Self {
                inner: NonNull::new_unchecked(ptr)
            }
        }
    }

    #[inline(always)]
    pub unsafe fn from_raw (ptr: *const AtomicUsize) -> Self {
        Self {
            inner: NonNull::new_unchecked(ptr as *mut AtomicUsize)
        }
    }

    #[inline(always)]
    pub fn count (&self) -> usize {
        unsafe {
            self.inner.as_ref().load(std::sync::atomic::Ordering::Acquire)
        }
    }

    #[inline(always)]
    pub fn into_raw (self) -> *const AtomicUsize {
        let this = ManuallyDrop::new(self);
        this.inner.as_ptr()
    }

    #[allow(unused)]
    #[inline(always)]
    pub unsafe fn increment_count (ptr: *const AtomicUsize) {
        let prev = (&*ptr).fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        debug_assert_ne!(prev, usize::MAX);
    }

    #[inline(always)]
    pub unsafe fn decrement_count (ptr: *const AtomicUsize) {
        let _ = Self::from_raw(ptr);
    }
}

impl Clone for AtomicCount {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            let prev = self.inner.as_ref().fetch_add(1, std::sync::atomic::Ordering::AcqRel);
            debug_assert_ne!(prev, usize::MAX);
        }

        Self { inner: self.inner }
    }
}

impl Drop for AtomicCount {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            let prev = self.inner.as_ref().fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
            debug_assert_ne!(prev, 0);

            if prev == 1 {
                let _ = Box::from_raw(self.inner.as_ptr());
            }
        }
    }
}

unsafe impl Send for AtomicCount {}
unsafe impl Sync for AtomicCount {}