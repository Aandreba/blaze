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
        match self.inner.binary_search_by_key(&evt.id(), |(x, _)| x.id()) {
            Ok(idx) => self.inner[idx].1.push(f),
            Err(idx) => self.inner.insert(idx, (evt.clone(), vec![f]))
        }
    }

    #[inline(always)]
    pub fn as_mut_queue (&mut self) -> &mut VecDeque<Entry> {
        &mut self.inner
    }
}