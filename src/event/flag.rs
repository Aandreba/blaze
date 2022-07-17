use std::{ptr::addr_of_mut, time::SystemTime};
use once_cell::sync::OnceCell;
use opencl_sys::{clSetUserEventStatus, CL_COMPLETE, clCreateUserEvent};

use super::{RawEvent, Event, ProfilingInfo};
use crate::{core::*, context::{RawContext, Global}};

/// Event that completes when the user specifies
#[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
#[derive(Debug, Clone)]
pub struct FlagEvent {
    inner: RawEvent,
    #[cfg(debug_assertions)]
    start: SystemTime,
    #[cfg(debug_assertions)]
    end: OnceCell<SystemTime>
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
            #[cfg(debug_assertions)]
            let start = SystemTime::now();

            if err != 0 { return Err(Error::from(err)); }
            
            Ok(Self {
                inner: RawEvent::from_id(id).unwrap(),
                #[cfg(debug_assertions)]
                start,
                #[cfg(debug_assertions)]
                end: OnceCell::new()
            })
        }
    }

    #[inline(always)]
    pub fn set_complete (&self, error: Option<ErrorType>) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            let end = SystemTime::now();
            let _ = self.end.set(end);
        }

        let status = error.map_or(CL_COMPLETE, Into::into);

        unsafe {
            tri!(clSetUserEventStatus(self.inner.id(), status));
        }

        Ok(())
    }

    #[inline(always)]
    pub fn into_inner (self) -> RawEvent {
        self.inner
    }
}

impl Event for FlagEvent {
    type Output = ();

    #[inline(always)]
    fn as_raw(&self) -> &RawEvent {
        &self.inner
    }

    #[inline(always)]
    fn consume (self, error: Option<Error>) -> Result<Self::Output> {
        if let Some(err) = error { return Err(err); }
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn profiling_nanos (&self) -> Result<ProfilingInfo<u64>> {
        if let Some(end) = self.end.get() {
            let start = self.start.duration_since(SystemTime::UNIX_EPOCH).map_err(|e| Error::new(ErrorType::InvalidValue, e))?.as_nanos();
            let end = end.duration_since(SystemTime::UNIX_EPOCH).map_err(|e| Error::new(ErrorType::InvalidValue, e))?.as_nanos();

            let start = u64::try_from(start).map_err(|e| Error::new(ErrorType::InvalidValue, e))?;
            let end = u64::try_from(end).map_err(|e| Error::new(ErrorType::InvalidValue, e))?;

            return Ok(ProfilingInfo {
                queued: start,
                submit: start,
                start: start,
                end: end,
                #[cfg(feature = "cl2")]
                complete: end
            })
        }

        Err(ErrorType::ProfilingInfoNotAvailable.into())
    }

    #[cfg(debug_assertions)]
    #[inline]
    fn profiling_time (&self) -> Result<ProfilingInfo<SystemTime>> {
        if let Some(end) = self.end.get() {
            return Ok(ProfilingInfo {
                queued: self.start,
                submit: self.start,
                start: self.start,
                end: *end,
                #[cfg(feature = "cl2")]
                complete: *end
            })
        }

        Err(ErrorType::ProfilingInfoNotAvailable.into())
    }

    #[cfg(debug_assertions)]
    #[inline(always)]
    fn duration (&self) -> Result<std::time::Duration> {
        if let Some(end) = self.end.get() {
            return end.duration_since(self.start).map_err(|e| Error::new(ErrorType::InvalidValue, e))
        }

        Err(ErrorType::ProfilingInfoNotAvailable.into())
    }
}