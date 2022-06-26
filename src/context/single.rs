use std::{borrow::Borrow, ptr::{addr_of_mut, addr_of}};
use opencl_sys::{cl_context, cl_command_queue, clCreateContext, clCreateCommandQueue, clReleaseContext};
use crate::{core::*};
use super::Context;

/// A simple RSCL context with a single command queue
pub struct SingleContext {
    ctx: cl_context,
    queue: cl_command_queue
}

impl SingleContext {
    pub fn new (device: impl Borrow<Device>) -> Result<Self> {
        let device = device.borrow();

        let mut err = 0;
        let errcode_addr = addr_of_mut!(err);

        let ctx; let queue;
        unsafe {
            ctx = clCreateContext(core::ptr::null_mut(), 1, addr_of!(device.0), None, core::ptr::null_mut(), errcode_addr);
            if err != 0 {
                return Err(Error::from(err));
            }

            queue = clCreateCommandQueue(ctx, device.0, 0, errcode_addr);
            if err != 0 {
                clReleaseContext(ctx);
                return Err(Error::from(err));
            }
        }

        Ok(Self { ctx, queue })
    }
}

impl Context for SingleContext {
    #[inline(always)]
    fn context_id (&self) -> cl_context {
        self.ctx
    }

    #[inline(always)]
    fn queue_count (&self) -> usize {
        1
    }

    #[inline(always)]
    fn next_queue (&self) -> cl_command_queue {
        self.queue
    }
}

unsafe impl Send for SingleContext {}
unsafe impl Sync for SingleContext {}