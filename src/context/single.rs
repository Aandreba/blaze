use std::{borrow::Borrow, ptr::{addr_of_mut, addr_of}};
use opencl_sys::{cl_context, cl_command_queue, clCreateContext, clReleaseContext, clRetainContext, clRetainCommandQueue, clReleaseCommandQueue};
use crate::{core::*};
use super::Context;



/// A simple RSCL context with a single command queue
pub struct SimpleContext {
    ctx: cl_context,
    queue: cl_command_queue
}

impl SimpleContext {
    pub fn new (device: impl Borrow<Device>) -> Result<Self> {
        let device : &Device = device.borrow();

        let mut err = 0;
        let errcode_addr = addr_of_mut!(err);

        unsafe {
            let ctx = clCreateContext(core::ptr::null_mut(), 1, addr_of!(device.0), None, core::ptr::null_mut(), errcode_addr);
            if err != 0 {
                return Err(Error::from(err));
            }

            #[allow(deprecated)]
            let queue = opencl_sys::clCreateCommandQueue(ctx, device.0, 0, errcode_addr);
            
            if err != 0 {
                clReleaseContext(ctx);
                return Err(Error::from(err));
            }

            Ok(Self { ctx, queue })
        }
    }
}

impl Context for SimpleContext {
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

impl Default for SimpleContext {
    #[inline(always)]
    fn default() -> Self {
        Self::new(Device::first().unwrap()).unwrap()
    }
}

impl Clone for SimpleContext {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainContext(self.ctx));
            tri_panic!(clRetainCommandQueue(self.queue))
        }

        Self { ctx: self.ctx, queue: self.queue }
    }
}

impl Drop for SimpleContext {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseContext(self.ctx));
            tri_panic!(clReleaseCommandQueue(self.queue))
        }
    }
}

unsafe impl Send for SimpleContext {}
unsafe impl Sync for SimpleContext {}