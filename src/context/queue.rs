use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, ops::{Deref, DerefMut}, marker::PhantomData};
use crate::{prelude::{RawCommandQueue, Result, Event, RawEvent}, event::consumer::*};

/// A command queue with extra capabilities to a raw OpenCL one.
#[derive(Debug, Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    pub(super) size: Arc<AtomicUsize>
}

impl CommandQueue {
    /// Creates a new command queue from a raw one.
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        Self {
            inner,
            size: Arc::new(AtomicUsize::default())
        }
    }

    /// Returns the current size of the queue.
    /// The size of the queue is defined as the number of enqueued events on it that haven't completed yet.
    /// Whilst this method is safe, it's result should be considered [ephemeral](https://en.wikipedia.org/wiki/Ephemerality).
    #[inline(always)]
    pub fn size (&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// Enqueues a new event without checking if the event's consumer has a safe lifetime.
    #[inline]
    pub unsafe fn enqueue_unchecked<'a, 'b, 'r: 'b, E: 'b + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>, C: 'a + Consumer> (&'r self, supplier: E, consumer: C) -> Result<Event<C>> {
        let inner = supplier(&self.inner)?;
        let evt = Event::new(inner, consumer);

        if self.size.fetch_add(1, Ordering::AcqRel) == usize::MAX {
            panic!("Queue size overflow");
        }

        let size = self.size.clone();
        if let Err(e) = evt.on_complete_silent(move |_, _| {
            size.fetch_sub(1, Ordering::AcqRel);
        }) {
            self.size.fetch_sub(1, Ordering::AcqRel);
            return Err(e);
        }

        return Ok(evt)
    }

    /// Enqueues a new phantom event without checking if the event's consumer has a safe lifetime.
    #[inline(always)]
    pub unsafe fn enqueue_phantom_unchecked<'a, 'r: 'a, T, E: 'a + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>> (&'r self, supplier: E) -> Result<PhantomEvent<T>> {
        self.enqueue_unchecked(supplier, PhantomData)
    }

    /// Enqueues a new event with a consumer with `'static` lifetime. 
    /// The `'static` lifetime ensures the compiler that the consumer is safe to be called at any time in the lifetime of the program.
    #[inline(always)]
    pub fn enqueue<'b, 'r: 'b, E: 'b + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>, C: 'static + Consumer> (&'r self, supplier: E, consumer: C) -> Result<Event<C>> {
        unsafe {
            self.enqueue_unchecked(supplier, consumer)
        }
    }

    /// Enqueues a new noop event.
    #[inline(always)]
    pub fn enqueue_noop<'a, 'r: 'a, E: 'a + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>> (&'r self, supplier: E) -> Result<NoopEvent> {
        self.enqueue(supplier, Noop)
    }

    /// Enqueues a new phantom event with a `'static` lifetime. The `'static` lifetime ensures the compiler that the consumer is safe to be called at any time in the lifetime of the program.
    #[inline(always)]
    pub fn enqueue_phantom<'a, 'r: 'a, T: 'static, E: 'a + FnOnce(&'r RawCommandQueue) -> Result<RawEvent>> (&'r self, supplier: E) -> Result<PhantomEvent<T>> {
        self.enqueue(supplier, PhantomData)
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