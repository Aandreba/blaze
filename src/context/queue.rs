use std::{sync::{atomic::{AtomicUsize, Ordering}}, ops::{Deref, DerefMut}, ptr::NonNull, alloc::Layout, num::NonZeroUsize, mem::ManuallyDrop};
use crate::{prelude::{RawCommandQueue, Result, Event, RawEvent}, event::consumer::*};

#[derive(Debug, Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    pub(super) size: Size
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        Self {
            inner,
            size: Size::new()
        }
    }

    #[inline(always)]
    pub fn size (&self) -> NonZeroUsize {
        unsafe {
            NonZeroUsize::new_unchecked(self.size.0.as_ref().load(Ordering::Relaxed))
        }
    }

    #[inline]
    pub unsafe fn enqueue_unchecked<'a, 'b, 'r: 'b, T, E: 'b + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>, C: Consumer<'a, T>> (&'r self, supplier: E, consumer: C) -> Result<Event<T, C>> {
        let inner = supplier(&self.inner)?;
        let evt = Event::new(inner, consumer);

        let size = self.size.clone();
        evt.on_complete(move |_, _| drop(size)).unwrap();

        return Ok(evt)
    }

    #[inline(always)]
    pub unsafe fn enqueue_noop_unchecked<'a, 'b, 'r: 'b, E: 'b + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>> (&'r self, supplier: E) -> Result<NoopEvent<'a>> {
        self.enqueue_unchecked(supplier, Noop::new())
    }

    #[inline(always)]
    pub fn enqueue<'b, 'r: 'b, T, E: 'b + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>, C: Consumer<'static, T>> (&'r self, supplier: E, consumer: C) -> Result<Event<T, C>> {
        unsafe {
            self.enqueue_unchecked(supplier, consumer)
        }
    }

    #[inline(always)]
    pub fn enqueue_noop<'b, 'r: 'b, E: 'b + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>> (&'r self, supplier: E) -> Result<NoopEvent<'static>> {
        self.enqueue(supplier, Noop::new())
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
pub(crate) struct Size (NonNull<AtomicUsize>);

impl Size {
    #[inline(always)]
    pub fn new () -> Size {
        unsafe {
            let alloc = std::alloc::alloc_zeroed(Layout::new::<AtomicUsize>());
            NonNull::new(alloc.cast()).map(Size).unwrap()
        }
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn drop_last (self) -> bool {
        let this = ManuallyDrop::new(self);

        unsafe {
            if this.0.as_ref().fetch_sub(1, Ordering::AcqRel) == 0 {
                std::alloc::dealloc(this.0.as_ptr().cast(), Layout::new::<AtomicUsize>());
                return true
            }
            return false
        }
    }
}

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