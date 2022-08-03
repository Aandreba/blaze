use std::{ptr::NonNull, alloc::{alloc, Layout}, sync::RwLock, num::NonZeroUsize, ops::{RangeBounds, Range}, cmp::Ordering};
use crate::{prelude::{RawEvent, Event}, event::EventStatus};
use super::{Flag, Listener, FALSE};
use elor::Either;

type Entry = (RawEvent, Vec<Listener>);

pub(super) struct ListenerQueue {
    inner: Vec<Entry>
}

impl ListenerQueue {
    pub const fn new () -> Self {
        Self {
            inner: Vec::new()
        }
    }

    #[inline]
    pub fn add_listener (&mut self, evt: &RawEvent, f: Listener) {
        match self.get_or_idx(&evt) {
            Either::Left((_, x)) => x.push(f),
            Either::Right(idx) => self.inner.insert(idx, (evt.clone(), vec![f]))
        }
    }

    #[inline]
    pub fn drain (&mut self, status: EventStatus) -> impl '_ + Iterator<Item = Entry> {
        self.inner.drain_filter(move |(x, _)| x.status().map_or(true, |x| x <= status))
    }
    
    /// pseudo binary search - O(log(n))
    fn get_or_idx (&mut self, target: &RawEvent) -> Either<&mut Entry, usize> {
        let left = 0usize;
        let right = self.inner.len();

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