use blaze_proc::docfg;
use crate::prelude::*;
use super::{Event};

#[cfg(feature = "cl1_1")]
use super::FlagEvent;

/// Event for [`EventExt::map`]
#[derive(Clone)]
pub struct Map<E, F> {
    pub(super) parent: E,
    pub(super) f: F
}

impl<T, E: Event, F: FnOnce(E::Output) -> T> Event for Map<E, F> {
    type Output = T;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.parent.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let v = self.parent.consume(err)?;
        Ok((self.f)(v))
    }
}

/// Event for [`EventExt::then`]
#[docfg(feature = "cl1_1")]
#[derive(Clone)]
pub struct Then<E, F> {
    pub(super) parent: E,
    pub(super) flag: FlagEvent,
    pub(super) f: F
}

#[cfg(feature = "cl1_1")]
impl<T, E: Event, F: FnOnce(E::Output) -> T> Event for Then<E, F> {
    type Output = T;

    #[inline(always)]
    fn as_raw (&self) -> &super::RawEvent {
        self.flag.as_raw()
    }

    #[inline(always)]
    fn parent_event (&self) -> &super::RawEvent {
        self.parent.as_raw()
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        match self.parent.consume(err) {
            Ok(x) => {
                let v = (self.f)(x);
                self.flag.set_complete(None)?;
                Ok(v)
            },

            Err(err) => {
                self.flag.set_complete(Some(err.ty))?;
                Err(err)
            }
        }
    }
}

/// Event for [`EventExt::inspect`]
#[derive(Debug, Clone)]
pub struct Inspect<E, F> {
    pub(super) parent: E,
    pub(super) f: F
}

impl<E: Event, F: FnOnce(&E::Output)> Event for Inspect<E, F> {
    type Output = E::Output;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.parent.as_raw()
    }

    #[inline]
    fn consume (self, err: Option<Error>) -> Result<Self::Output> {
        let v = self.parent.consume(err)?;
        (self.f)(&v);
        Ok(v)
    }
} 