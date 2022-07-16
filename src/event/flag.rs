use std::{ptr::addr_of_mut};
use opencl_sys::{clSetUserEventStatus, CL_COMPLETE, clCreateUserEvent};

use super::{RawEvent, Event};
use crate::{core::*, context::{RawContext, Global}};

/// Event that completes when the user specifies
#[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct FlagEvent (RawEvent);

impl FlagEvent {
    #[inline(always)]
    pub fn new () -> Result<Self> {
        Self::new_in(&Global)
    }

    #[inline]
    pub fn new_in (ctx: &RawContext) -> Result<Self> {
        let mut err = 0;

        unsafe {
            let id = clCreateUserEvent(ctx.id(), addr_of_mut!(err));
            if err != 0 { return Err(Error::from(err)); }
            Ok(Self(RawEvent::from_id(id).unwrap()))
        }
    }

    #[inline(always)]
    pub fn set_complete (&self, error: Option<ErrorType>) -> Result<()> {
        let status = error.map_or(CL_COMPLETE, Into::into);

        unsafe {
            tri!(clSetUserEventStatus(self.0.id(), status));
        }

        Ok(())
    }

    #[inline(always)]
    pub fn into_inner (self) -> RawEvent {
        self.0
    }
}

impl Event for FlagEvent {
    type Output = ();

    #[inline(always)]
    fn as_raw(&self) -> &RawEvent {
        &self.0
    }

    #[inline(always)]
    fn consume (self, error: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = error { return Err(err); }
        Ok(())
    }
}