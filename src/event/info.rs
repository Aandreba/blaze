use std::time::SystemTime;
use opencl_sys::*;
use rscl_proc::docfg;
use super::RawEvent;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct ProfilingInfo {
    pub queued: SystemTime,
    pub submit: SystemTime,
    pub start: SystemTime,
    pub end: SystemTime,
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
    #[cfg(feature = "cl2")]
    pub complete: SystemTime,
}

impl ProfilingInfo {
    pub fn new (event: &RawEvent) -> Result<Self> {

    }

    #[inline]
    fn get_info (event: &RawEvent) -> SystemTime {
        let mut value = 0;
        unsafe {
            tri!(clGetEventProfilingInfo())
        }
    }
}