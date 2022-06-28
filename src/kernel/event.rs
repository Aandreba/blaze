use std::ptr::addr_of_mut;
use opencl_sys::{clEnqueueNDRangeKernel};
use parking_lot::lock_api::RawMutex;

use crate::{event::{RawEvent, Event}, context::Context};
use super::Build;
use crate::core::*;

pub struct NdKernelEvent {
    raw: RawEvent,
}

impl NdKernelEvent {
    pub fn new<C: Context, const N: usize> (builder: &Build<'_, C, N>) -> Result<Self> {
        let work_dim = u32::try_from(N).unwrap();
        let (num_events_in_wait_list, event_wait_list) = (0, core::ptr::null()); // todo
        let global_work_dims = builder.global_work_dims.as_ptr();
        let local_work_dims = match builder.local_work_dims {
            Some(x) => x.as_ptr(),
            None => core::ptr::null()
        };

        builder.parent.lock.lock();

        unsafe { 
            for i in 0..builder.args.len() {
                if let Some(ref arg) = builder.args[i] {
                    if let Err(e) = arg.set_argument(i as u32, builder.parent) {
                        builder.parent.lock.unlock();
                        return Err(e);
                    }

                    continue
                }

                builder.parent.lock.unlock();
                return Err(Error::InvalidArgValue)
            }

            let mut event = core::ptr::null_mut();
            let err = clEnqueueNDRangeKernel(builder.parent.ctx.next_queue(), builder.parent.inner, work_dim, core::ptr::null(), global_work_dims, local_work_dims, num_events_in_wait_list, event_wait_list, addr_of_mut!(event));
            
            builder.parent.lock.unlock();
            if err != 0 { return Err(Error::from(err)); }
            let raw = RawEvent::from_ptr(event);
            Ok(Self { raw })
        }
    }
}

impl Event for NdKernelEvent {
    type Output = ();

    #[inline(always)]
    fn consume (self) -> Self::Output {
        ()
    }
}

impl AsRef<RawEvent> for NdKernelEvent {
    #[inline(always)]
    fn as_ref(&self) -> &RawEvent {
        &self.raw
    }
}