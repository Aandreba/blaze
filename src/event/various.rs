use rscl_proc::docfg;
use super::{Event};

#[cfg(feature = "cl1_1")]
use super::FlagEvent;

#[docfg(feature = "cl1_1")]
#[derive(Clone)]
pub struct Map<E, F> {
    pub(super) parent: E,
    pub(super) flag: FlagEvent,
    pub(super) f: F
}

#[cfg(feature = "cl1_1")]
impl<T, E: Event, F: FnOnce(E::Output) -> T> Event for Map<E, F> {
    type Output = T;

    #[inline(always)]
    fn as_raw (&self) -> &super::RawEvent {
        self.flag.as_raw()
    }

    #[inline(always)]
    fn consume (self) -> Self::Output {
        (self.f)(self.parent.consume())
    }
}