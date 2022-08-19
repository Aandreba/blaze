use utils_atomics::FillQueue;
use crate::prelude::{Result, RawCommandQueue, RawEvent};
use super::{Context, Global};

pub struct Scope<'a, C: ?Sized + Context = Global> {
    ctx: &'a C,
    pub(super) events: FillQueue<RawEvent>
}

impl<'a, C: ?Sized + Context> Scope<'a, C> {
    #[inline(always)]
    pub fn enqueue<F: FnOnce(&RawCommandQueue) -> Result<RawEvent>> (&'a self, f: F) -> Result<RawEvent> {
        let evt = self.ctx.enqueue(f)?;
        self.events.push(evt.clone());
        Ok(evt)
    }
}