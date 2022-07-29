flat_mod!(eventual);

use std::{sync::{Arc, atomic::{AtomicUsize}}, ops::{Deref, DerefMut}, ptr::{addr_of_mut}};
use crate::prelude::{RawCommandQueue, WaitList, Result, RawBuffer, RawEvent, Event};
use crossbeam::atomic::AtomicConsume;
use opencl_sys::*;

#[derive(Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    #[cfg(feature = "cl1_1")]
    size: Arc<AtomicUsize>
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        CommandQueue {
            inner,
            #[cfg(feature = "cl1_1")]
            size: Arc::new(AtomicUsize::new(0))
        }
    }

    #[inline(always)]
    pub fn size (&self) -> usize {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_1")] {
                self.size.load_consume()
            } else {
                0
            }
        }
    }
}

#[cfg(feature = "cl1_1")]
impl CommandQueue {
    pub fn enqueue<F: 'static + Send + FnOnce(&RawCommandQueue, WaitList) -> Result<RawEvent>> (&self, wait: impl Into<WaitList>, f: F) -> Result<Eventual> {
        self.size.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        let eventual = Eventual::new();
        let wait : WaitList = wait.into();
        
        if wait.is_empty() {
            Self::final_enqueue(f(&self.inner, wait), self.size.clone(), eventual.clone());
            return Ok(eventual);
        };
        
        #[cfg(feature = "cl1_2")]
        let marker = self.marker(wait.clone())?;
        #[cfg(not(feature = "cl1_2"))]
        let marker = RawEvent::join_in(&self.context()?, wait.iter().cloned())?.to_raw();

        let raw = self.inner.clone();
        let marker_eventual = eventual.clone();
        let marker_size = self.size.clone();

        marker.on_complete(move |_, res| unsafe {
            if let Err(err) = res {
                marker_size.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
                marker_eventual.set_unchecked(Err(err));
                return;
            }

            let evt = f(&raw, wait);
            Self::final_enqueue(evt, marker_size, marker_eventual);
        })?;

        todo!()
    }

    pub unsafe fn enqueue_read_buffer<T: 'static + Sync> (&self, buffer: RawBuffer, offset: usize, size: usize, ptr: *mut T, wait: impl Into<WaitList>) -> Result<Eventual> {
        let ptr = AssertSend::new_unchecked(ptr);

        let f = move |queue: &RawCommandQueue, wait: WaitList| {
            let (num_events_in_wait_list, event_wait_list) = wait.raw_parts();
            let mut evt = core::ptr::null_mut();
            tri!(clEnqueueReadBuffer(queue.id(), buffer.id(), CL_TRUE, offset as _, size as _, ptr.into_inner().cast(), num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
            Ok(RawEvent::from_id(evt).unwrap())
        };

        self.enqueue(wait, f)
    }

    fn final_enqueue (evt: Result<RawEvent>, marker_size: Arc<AtomicUsize>, marker_eventual: Eventual) {
        // If evt exists, tell it to inform us when it's done. Othwerwise, decrease counter now.
        if let Ok(ref evt) = evt {
            let evt_size = marker_size.clone();
            println!("prev: {:?}", evt.reference_count()); 

            if evt.on_complete(move |evt, _| {
                println!("in: {:?}", evt.reference_count()); 
                evt_size.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
            }).is_err() {
                marker_size.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
            }
            println!("next: {:?}", evt.reference_count()); 
        } else {
            marker_size.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        }

        unsafe {
            marker_eventual.set_unchecked(evt)
        }
    }
}

#[cfg(not(feature = "cl1_1"))]
impl CommandQueue {
    pub unsafe fn enqueue_read_buffer (&self, wait: impl Into<WaitList>) -> Result<()> {
        todo!()
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

#[cfg(feature = "cl1_1")]
struct AssertSend<T> (T);

impl<T> AssertSend<T> {
    #[inline(always)]
    pub fn new (t: T) -> Self where T: Send {
        unsafe {
            Self::new_unchecked(t)
        }
    }

    #[inline(always)]
    pub unsafe fn new_unchecked (t: T) -> Self {
        Self(t)
    }

    #[inline(always)]
    pub fn into_inner (self) -> T {
        self.0
    }
}

impl<T> Deref for AssertSend<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for AssertSend<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl<T> Send for AssertSend<T> {}

#[cfg(test)]
mod test {
    use std::ptr::NonNull;
    use crate::{prelude::*, context::{CommandQueue, SimpleContext}};

    #[test]
    fn test () -> Result<()> {
        let ctx = SimpleContext::default()?;
        let queue = CommandQueue::new(ctx.next_queue().clone());

        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let buffer = RawBuffer::new_in(&ctx, core::mem::size_of_val(&data), MemFlags::new(MemAccess::default(), HostPtr::COPY), NonNull::new(data.as_ptr() as *mut _))?;

        let mut read = [0; 10];
        let evtual = unsafe {
            queue.enqueue_read_buffer(buffer, 0, core::mem::size_of_val(&data), read.as_mut_ptr(), EMPTY)
        }?;

        let evt = evtual.wait()?;
        
        drop(evt);
        println!("dropped event");

        drop(evtual);
        println!("dropped eventual");

        drop(queue);
        println!("dropped queue");

        drop(ctx);
        println!("dropped context");

        Ok(())    
    }
}