use std::sync::{Arc, atomic::AtomicUsize};
use crate::prelude::*;

flat_mod!(eventual);

pub struct CommandQueue {
    inner: RawCommandQueue,
    #[cfg(feature = "cl1_2")]
    size: Arc<AtomicUsize>
}

impl CommandQueue {
    #[inline(always)]
    pub fn new (inner: RawCommandQueue) -> Self {
        Self { 
            inner,
            #[cfg(feature = "cl1_2")]
            size: Arc::new(AtomicUsize::default())
        }
    }

    #[cfg(feature = "cl1_2")]
    pub fn enqueue<F: 'static + Send + FnOnce(RawCommandQueue, WaitList)> (&self, f: F, wait: impl Into<WaitList>) -> Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                let prev = self.size.fetch_add(1, std::sync::atomic::Ordering::Release);
                if prev == usize::MAX {
                    panic!("CommandQueue overflow");
                }
            } else {
                self.size.fetch_add(1, std::sync::atomic::Ordering::Release);
            }
        }

        let wait : WaitList = wait.into();
        if wait.is_empty() {
            todo!()
        }

        let marker = match self.inner.marker(wait.clone()) {
            Ok(x) => x,
            Err(e) => {
                self.size.fetch_sub(1, std::sync::atomic::Ordering::Release);
                return Err(e)
            }
        };

        marker.on_complete(move |a, b| {
            
        });

        todo!();
    }
}