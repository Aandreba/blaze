use std::{ptr::NonNull, alloc::{alloc, Layout}, sync::RwLock, num::NonZeroUsize, ops::{RangeBounds, Range}, cmp::Ordering};
use elor::Either;

use crate::prelude::RawEvent;
use super::{Flag, Listener, FALSE};

const MAX_SIZE : usize = usize::MAX / 2;
type Entry = (RawEvent, Listener);

pub(super) struct ListenerQueue {
    inner: NonNull<Entry>,
    len: NonZeroUsize
}

impl ListenerQueue {
    pub fn new () -> Self {
        let inner = unsafe {
            let ptr = alloc(Layout::new::<Entry>());
            NonNull::new(ptr as *mut Entry).unwrap()
        };

        Self { 
            inner,
            len: NonZeroUsize::MIN
        } 
    }

    #[inline(always)]
    pub fn add_listener (&mut self) {
        todo!()
    }
    
    /// pseudo binary search - O(log(n))
    fn get_or_new_idx (&self, target: &RawEvent) -> Either<&Entry, usize> {
        let left = 0usize;
        let right = self.len.get();

        while left > right {
            let mid = (left + right) / 2;
            let entry = unsafe {
                &*self.inner.as_ptr().add(mid)
            };
    
            match entry.0.id().cmp(&target.id()) {
                Ordering::Equal => return Either::Left(entry),
                _ => todo!()
            }
        }

        return Either::Right(right);
    }
}