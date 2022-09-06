use std::{sync::{atomic::{AtomicUsize, Ordering}}, ops::{Deref, DerefMut}, ptr::NonNull, alloc::Layout, num::NonZeroUsize};
use crate::prelude::{RawCommandQueue, Result, Event, RawEvent};

#[derive(Debug, Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    size: Size
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        let size = unsafe {
            let alloc = std::alloc::alloc_zeroed(Layout::new::<AtomicUsize>());
            NonNull::new(alloc.cast()).map(Size).unwrap()
        };

        Self {
            inner,
            size
        }
    }

    #[inline(always)]
    pub fn size (&self) -> NonZeroUsize {
        unsafe {
            NonZeroUsize::new_unchecked(self.size.0.as_ref().load(Ordering::Relaxed))
        }
    }

    #[inline(always)]
    pub fn enqueue<'a, 'b, 'r: 'b, T, E: 'b + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>, F: 'a + FnOnce() -> Result<T>> (&'r self, supplier: E, f: F) -> Result<Event<'a, T>> {
        let inner = supplier(&self.inner)?;
        let evt = Event::new(inner, f);

        let size = self.size.clone();
        evt.on_complete(move |_, _| drop(size));

        return Ok(evt)
    }

    #[inline(always)]
    pub fn enqueue_noop<'a, 'b, 'r: 'b, E: 'b + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>> (&'r self, supplier: E) -> Result<Event<'a, ()>> {
        self.enqueue(supplier, || Ok(()))
    }
}

impl Deref for CommandQueue {
    type Target = RawCommandQueue;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CommandQueue {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct Size (NonNull<AtomicUsize>);

impl Clone for Size {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            self.0.as_ref().fetch_add(1, Ordering::AcqRel);
        }
        Self(self.0)
    }
}

impl Drop for Size {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            if self.0.as_ref().fetch_sub(1, Ordering::AcqRel) == 0 {
                std::alloc::dealloc(self.0.as_ptr().cast(), Layout::new::<AtomicUsize>())
            }
        }
    }
}

unsafe impl Send for Size {}
unsafe impl Sync for Size {}