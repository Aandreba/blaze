flat_mod!(supplier, eventual);

use std::{sync::{Arc, atomic::{AtomicUsize}}, ops::{Deref, DerefMut}};
use crate::{prelude::{RawCommandQueue, RawEvent, Result, Kernel}, event::{WaitList}};


/// A smart command queue. Events pushed to this queue will not be pushed to it's OpenCL counterpart until all
/// their dependants (a.k.a the events in the wait list) have completed.
#[derive(Debug, Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    #[cfg(feature = "cl1_1")]
    size: Arc<AtomicUsize>
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        Self { 
            inner,
            #[cfg(feature = "cl1_1")]
            size: Arc::new(AtomicUsize::default())
        }
    }

    #[inline(always)]
    pub fn size (&self) -> usize {
        #[cfg(feature = "cl1_1")]
        return self.size.load(std::sync::atomic::Ordering::Relaxed);
        #[cfg(not(feature = "cl1_1"))]
        0
    }

    #[cfg(feature = "cl1_1")]
    #[inline]
    pub fn enqueue<F: EventSupplier> (&self, f: F, wait: impl Into<WaitList>) -> Result<Eventual<F>> {
        self.size.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let wait : WaitList = wait.into();
        let ctx = match self.inner.context() {
            Ok(x) => x,
            Err(e) => {
                self.size.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                return Err(e)
            }  
        };

        let join = match <RawEvent as crate::prelude::EventExt>::join_in(&ctx, wait.iter().cloned()) {
            Ok(x) => x,
            Err(e) => {
                self.size.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                return Err(e)
            }
        };

        Ok(Eventual::new(join, self.clone(), f, wait))
    }

    #[cfg(not(feature = "cl1_1"))]
    #[inline(always)]
    pub fn enqueue<F: EventSupplier> (&self, f: F, wait: impl Into<WaitList>) -> Result<Eventual<F>> {
        Ok(Eventual::new(&self.inner, f,))
    }

    #[inline(always)]
    pub unsafe fn enqueue_kernel<K: DerefMut<Target = Kernel>, const N: usize> (&self, kernel: K, global_work_dims: [usize;N], local_work_dims: impl Into<Option<[usize;N]>>, wait: impl Into<WaitList>) -> Result<Eventual<KernelEnqueueSupplier<K, N>>> {
        let f = KernelEnqueueSupplier {
            kernel, global_work_dims,
            local_work_dims: local_work_dims.into()
        };

        self.enqueue(f, wait)
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