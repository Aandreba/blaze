use std::{num::{NonZeroUsize}};
use blaze_rs::prelude::*;

lazy_static! {
    static ref BLASE_CTX: BlaseContext = BlaseContext::new().unwrap();
}

#[derive(Debug, Clone, Copy)]
pub struct BlaseContext {
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

        Err(Error::new(ErrorType::InvalidValue, "No queues found"))
    }
}

#[inline(always)]
pub fn work_group_size (n: usize) -> usize {
    BLASE_CTX.max_wgs.get().min(n)
}