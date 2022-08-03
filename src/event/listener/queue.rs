use std::{ptr::NonNull, alloc::{alloc, Layout}, sync::{RwLock, atomic::AtomicPtr}, num::NonZeroUsize, ops::{RangeBounds, Range}, cmp::Ordering, collections::VecDeque};
use crate::{prelude::{RawEvent, Event}, event::EventStatus};
use super::{Flag, Listener, FALSE};
use crossbeam::atomic::AtomicCell;
use elor::Either;
use once_cell::sync::Lazy;

type Entry = (RawEvent, Vec<Listener>);

pub(super) struct ListenerQueue {
    inner: Lazy<VecDeque<Entry>>
}

impl ListenerQueue {
    pub const fn new () -> Self {
        Self {
            inner: Lazy::new(VecDeque::new)
        }
    }

    #[inline]
    pub fn add_listener (&mut self, evt: &RawEvent, f: Listener) {
        match self.get_or_idx(&evt) {
            Either::Left((_, x)) => x.push(f),
            Either::Right(idx) => self.inner.insert(idx, (evt.clone(), vec![f]))
        }
    }

    #[inline(always)]
    pub fn as_mut_queue (&mut self) -> &mut VecDeque<Entry> {
        &mut self.inner
    }
    
    /// pseudo binary search - O(log(n))
    fn get_or_idx (&mut self, target: &RawEvent) -> Either<&mut Entry, usize> {
        let mut left = 0usize;
        let mut right = self.inner.len();

        while left < right {
            let mid = (left + right).div_ceil(2);
            let entry = &mut self.inner[mid];
    
            match entry.0.id().cmp(&target.id()) {
                Ordering::Equal => return Either::Left(entry),
                Ordering::Less => left = mid - 1,
                Ordering::Greater => right = mid
            }
        }

        return Either::Right(right);
    }
}