use std::sync::Arc;
use crate::prelude::{Result, ErrorType};
use super::{Event, FlagEvent};

/// Event for [`EventExt::abortable`](super::EventExt::abortable)
#[derive(Clone)]
pub struct Abortable<E> {
    inner: E,
    flag: Arc<FlagEvent>
}

impl<E: Event> Abortable<E> {
    #[inline(always)]
    pub fn new (inner: E) -> Result<(Self, AbortHandle)> {
        let flag = FlagEvent::new().map(Arc::new)?;
        let handler = AbortHandle(flag.clone());
        
        let flag2 = flag.clone();
        inner.as_raw().on_complete(move |_, status| {
            let status = status.map_err(|e| e.ty).err();
            let _ = flag2.set_complete(status);
        })?;

        Ok((Self { inner, flag }, handler))
    }
}

impl<E: Event> Event for Abortable<E> {
    type Output = Option<E::Output>;

    #[inline(always)]
    fn as_raw (&self) -> &super::RawEvent {
        self.flag.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<crate::prelude::Error>) -> Result<Self::Output> {
        match err {
            Some(err) if err.ty == ErrorType::Aborted => Ok(None),
            err => self.inner.consume(err).map(Some)
        }
    }
}

#[derive(Clone)]
pub struct AbortHandle (Arc<FlagEvent>);

impl AbortHandle {
    #[inline(always)]
    pub fn abort (self) -> Result<()> {
        self.0.set_complete(Some(ErrorType::Aborted))
    }
}