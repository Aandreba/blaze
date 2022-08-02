use std::{sync::Arc, fmt::Debug, hash::Hash};
use crate::prelude::*;

#[derive(Clone)]
pub struct CommandQueue {
    inner: RawCommandQueue,
    #[cfg(feature = "cl1_1")]
    size: Arc<()>
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        Self { 
            inner,
            #[cfg(feature = "cl1_1")]
            size: Arc::new(())
        }
    }

    #[inline(always)]
    pub fn size (&self) -> usize {
        Arc::strong_count(&self.size) - 1
    }

    #[cfg(feature = "cl1_1")]
    #[inline]
    pub fn enqueue<F: 'static + Send + FnOnce(&RawCommandQueue, WaitList) -> Result<RawEvent>> (&self, f: F, wait: impl Into<WaitList>) -> Result<RawEvent> {
        let size = self.size.clone();
        let evt = f(&self.inner, wait.into())?;
        let size = Arc::into_raw(size);

        unsafe {
            if let Err(e) = evt.on_complete_raw(decrease_count, size as *mut std::ffi::c_void) {
                Arc::decrement_strong_count(size);
                return Err(e)
            }
        }

        Ok(evt)
    }

    #[cfg(not(feature = "cl1_1"))]
    #[inline(always)]
    pub fn enqueue<F: 'static + Send + FnOnce(&RawCommandQueue, WaitList) -> Result<RawEvent>> (&self, f: F, wait: impl Into<WaitList>) -> Result<RawEvent> {
        f(&self.inner, wait.into())
    }
}

impl Debug for CommandQueue {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandQueue")
            .field("inner", &self.inner)
            .field("size", &self.size())
            .finish()
    }
}

impl PartialEq for CommandQueue {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Hash for CommandQueue {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl Eq for CommandQueue {}

#[doc(hidden)]
#[cfg(feature = "cl1_2")]
unsafe extern "C" fn decrease_count (_event: opencl_sys::cl_event, _event_command_status: opencl_sys::cl_int, user_data: *mut std::ffi::c_void) {
    Arc::decrement_strong_count(user_data as *mut ());
}