use std::{ops::{Deref, DerefMut}, ptr::addr_of_mut, mem::MaybeUninit, num::NonZeroU32};
use opencl_sys::*;
use crate::{memobj::MemObject, prelude::*, buffer::flags::{MemFlags, HostPtr, MemAccess}};

#[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Pipe (MemObject);

impl Pipe {
    #[inline(always)]
    pub fn new (access: MemAccess, host_access: bool, packet_size: u32, max_packets: u32) -> Result<Self> {
        Self::new_in(&Global, access, host_access, packet_size, max_packets)
    }

    pub fn new_in (ctx: &RawContext, access: MemAccess, host_access: bool, packet_size: u32, max_packets: u32) -> Result<Self> {
        let flags = MemFlags::with_host_access(access, MemAccess::new(host_access, host_access), HostPtr::default());

        let mut err = 0;
        let id = unsafe {
            clCreatePipe(ctx.id(), flags.to_bits(), packet_size, max_packets, core::ptr::null_mut(), addr_of_mut!(err))
        };

        if err != 0 { return Err(Error::from(err)); }
        Ok(Self(MemObject::from_id(id).unwrap()))
    }

    /// Return pipe packet size specified when pipe is created.
    #[inline(always)]
    pub fn packet_size (&self) -> Result<NonZeroU32> {
        self.get_info(CL_PIPE_PACKET_SIZE)
    }

    /// Return max. number of packets specified when pipe is created.
    #[inline(always)]
    pub fn max_packets (&self) -> Result<NonZeroU32> {
        self.get_info(CL_PIPE_MAX_PACKETS)
    }

    fn get_info<T> (&self, ty: cl_pipe_info) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();
        
        unsafe {
            tri!(clGetPipeInfo(self.id(), ty, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

impl Deref for Pipe {
    type Target = MemObject;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Pipe {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}