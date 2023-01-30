use std::{num::{NonZeroUsize}};
use blaze_rs::prelude::*;

thread_local! {
    static BLASE_CTX: BlaseContext = BlaseContext::new().unwrap();
}

#[derive(Debug, Clone, Copy)]
struct BlaseContext {
    max_wgs: NonZeroUsize,
}

impl BlaseContext {
    #[inline]
    fn new () -> Result<Self> {
        let mut max_wgs = None;
        for queue in Global.queues() {
            let device = queue.device()?;
            let wgs = device.max_work_group_size()?;

            max_wgs = match max_wgs {
                Some(x) => Some(wgs.min(x)),
                None => Some(wgs)
            };
        }

        if let Some(max_wgs) = max_wgs {
            return Ok(Self { max_wgs })
        }

        Err(Error::new(ErrorKind::InvalidValue, "No queues found"))
    }
}

#[inline(always)]
pub fn max_work_group_size () -> NonZeroUsize {
    BLASE_CTX.with(|ctx| ctx.max_wgs)
}

#[inline(always)]
pub fn work_group_size (n: usize) -> usize {
    max_work_group_size().get().min(n)
}