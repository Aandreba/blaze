use std::{ptr::addr_of_mut, ops::Deref};
use opencl_sys::*;
use crate::prelude::*;

/// User event that's completed manually.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FlagEvent {
    inner: RawEvent
}

impl FlagEvent {
    /// Creates a new [`FlagEvent`] with the [`Global`] context
    #[inline(always)]
    pub fn new () -> Result<Self> {
        Self::new_in(&Global)
    }

    /// Creates a new [`FlagEvent`] with the specified context.
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

    /// Converts the [`FlagEvent`] into it's inner [`RawEvent`]
    #[inline(always)]
    pub fn into_inner (self) -> RawEvent {
        self.inner
    }

    /// Attempts to mark the event as completed, returning `true` if successful and `false` if the event was already completed.
    #[inline(always)]
    pub fn try_mark (&self, error: Option<ErrorCode>) -> Result<bool> {
        let status = error.map_or(CL_COMPLETE, ErrorCode::as_i32);

        unsafe {
            match clSetUserEventStatus(self.inner.id(), status) {
                CL_SUCCESS => Ok(true),
                CL_INVALID_OPERATION => Ok(false),
                other => Err(other.into())
            }
        }
    }

    /// Subscribes to the event.
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