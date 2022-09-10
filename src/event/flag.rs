use std::{ptr::addr_of_mut, ops::Deref};
use opencl_sys::*;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FlagEvent {
    inner: RawEvent
}

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
            
            return Ok(Self {
                inner: RawEvent::from_id(id).unwrap(),
            })
        }
    }

    #[inline(always)]
    pub fn into_inner (self) -> RawEvent {
        self.inner
    }

    #[inline(always)]
    pub fn try_mark (&self, error: Option<ErrorType>) -> Result<bool> {
        let status = error.map_or(CL_COMPLETE, Into::into);

        unsafe {
            match clSetUserEventStatus(self.inner.id(), status) {
                CL_SUCCESS => Ok(true),
                CL_INVALID_OPERATION => Ok(false),
                other => Err(other.into())
            }
        }
    }

    #[inline(always)]
    pub fn subscribe (&self) -> RawEvent {
        self.inner.clone()
    }
}

impl Deref for FlagEvent {
    type Target = RawEvent;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}