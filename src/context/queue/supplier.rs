use std::ops::DerefMut;
use crate::{prelude::{RawCommandQueue, Result, RawEvent, Kernel}, event::WaitList};

pub trait EventSupplier {
    fn supply (self, queue: &RawCommandQueue, wait: WaitList) -> Result<RawEvent>;
}

impl<F: FnOnce(&RawCommandQueue, WaitList) -> Result<RawEvent>> EventSupplier for F {
    #[inline(always)]
    fn supply (self, queue: &RawCommandQueue, wait: WaitList) -> Result<RawEvent> {
        self(queue, wait)
    }
}

pub struct KernelEnqueueSupplier<K, const N: usize> {
    kernel: K,
    global_work_dims: [usize; N], 
    local_work_dims: Option<[usize; N]>
}

impl<K: DerefMut<Target = Kernel>, const N: usize> EventSupplier for KernelEnqueueSupplier<K, N> {
    #[inline(always)]
    fn supply (self, queue: &RawCommandQueue, wait: WaitList) -> Result<RawEvent> {
        unsafe { self.kernel.enqueue_with_queue(queue, self.global_work_dims, self.local_work_dims, wait) }
    }
}