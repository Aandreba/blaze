use box_iter::BoxIntoIter;
use crate::prelude::{Result, RawCommandQueue, Global, Context};
use super::{RawEvent, Event, WaitList};

#[derive(Debug, Clone)]
pub struct Join<E> {
    marker: RawEvent,
    evts: Box<[E]>
}

impl<E: Event> Join<E> {
    #[inline(always)]
    pub fn new<I: IntoIterator<Item = E>> (iter: I) -> Result<Self> {
        Self::new_in(iter, Global.next_queue())
    }

    #[inline]
    pub fn new_in<I: IntoIterator<Item = E>> (iter: I, queue: &RawCommandQueue) -> Result<Self> {
        let (raw, evts) : (Vec<_>, Vec<_>) = iter.into_iter().map(|x| (x.to_raw(), x)).unzip();
        let marker = queue.marker(WaitList::new(raw))?;
        Ok(Self { marker, evts: evts.into_boxed_slice() })
    }
}

impl<E: Event> Event for Join<E> {
    type Output = Vec<E::Output>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        &self.marker
    }

    #[inline]
    fn consume (self, err: Option<crate::prelude::Error>) -> Result<Self::Output> {
        let mut result = Vec::with_capacity(self.evts.len());

        for evt in self.evts.into_iter() {
            let v = evt.consume(err.clone())?;
            result.push(v)
        }

        Ok(result)
    }
}