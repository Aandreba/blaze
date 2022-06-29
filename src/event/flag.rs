use std::{ptr::addr_of_mut};
use opencl_sys::{clSetUserEventStatus, CL_COMPLETE, clCreateUserEvent};

use super::{RawEvent, Event};
use crate::{core::*, context::{Context, Global}};

#[derive(Clone)]
pub struct FlagEvent<C: Context = Global> (RawEvent, C);

impl FlagEvent {
    #[inline(always)]
    pub fn new () -> Result<Self> {
        Self::new_in(Global)
    }
}

impl<C: Context> FlagEvent<C> {
    pub fn new_in (ctx: C) -> Result<Self> {
        let mut err = 0;

        unsafe {
            let id = clCreateUserEvent(ctx.context_id(), addr_of_mut!(err));
            if err != 0 { return Err(Error::from(err)); }
            Ok(Self(RawEvent::from_ptr(id), ctx))
        }
    }

    #[inline(always)]
    pub fn set_complete (&self, error: Option<Error>) -> Result<()> {
        let status = error.map_or(CL_COMPLETE, Into::into);

        unsafe {
            tri!(clSetUserEventStatus(self.0.0, status));
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
    fn consume (self) -> Self::Output {
        ()
    }
}

impl AsRef<RawEvent> for FlagEvent {
    #[inline(always)]
    fn as_ref(&self) -> &RawEvent {
        &self.0
    }
}