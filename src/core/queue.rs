use super::*;
use opencl_sys::{cl_command_queue, clRetainCommandQueue, clReleaseCommandQueue, clFlush, clFinish};

#[repr(transparent)]
pub struct CommandQueue (pub(crate) cl_command_queue);

impl CommandQueue {
    #[inline(always)]
    pub const fn id (&self) -> cl_command_queue {
        self.0
    }

    #[inline(always)]
    pub fn flush (&self) -> Result<()> {
        unsafe {
            tri!(clFlush(self.0));
        }

        Ok(())
    }

    #[inline(always)]
    pub fn finish (&self) -> Result<()> {
        unsafe {
            tri!(clFinish(self.0));
        }

        Ok(())
    }
}

impl Clone for CommandQueue {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainCommandQueue(self.0))
        }

        Self(self.0)
    }
}

impl Drop for CommandQueue {
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseCommandQueue(self.0))
        }
    }
}