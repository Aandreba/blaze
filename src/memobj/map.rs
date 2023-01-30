use std::ptr::addr_of_mut;

use opencl_sys::*;
use crate::prelude::{MemAccess, Context, Global, RawEvent};
use super::RawMemObject;

pub(crate) struct MapPtr<T, C: Context = Global> {
    pub ptr: *mut [T],
    mem: RawMemObject,
    ctx: C
}

impl<T, C: Context> MapPtr<T, C> {
    #[inline(always)]
    pub fn new (ptr: *mut [T], mem: RawMemObject, ctx: C) -> Self {
        Self {
            ptr,
            mem,
            ctx
        }
    }
}

impl<T, C: Context> Drop for MapPtr<T, C> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.ctx.next_queue().enqueue_noop(|queue| {
                let mut event = core::ptr::null_mut();
                tri!(clEnqueueUnmapMemObject(queue.id(), self.mem.id(), self.ptr.cast(), 0, core::ptr::null(), addr_of_mut!(event)));
                return Ok(RawEvent::from_id(event).unwrap())
            }).unwrap().join_unwrap();
        }
    }
}

unsafe impl<T: Send, C: Send + Context> Send for MapPtr<T, C> {}
unsafe impl<T: Sync, C: Sync + Context> Sync for MapPtr<T, C> {}

/// Mapping flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapFlags {
    Access (MemAccess),
    /// This flag specifies that the region being mapped in the memory object is being mapped for writing. The contents of the region being mapped are to be discarded. This is typically the case when the region being mapped is overwritten by the host.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_2")))]
    #[cfg(feature = "cl1_2")]
    WriteInvalidate
}

impl MapFlags {
    #[inline]
    pub const fn from_bits (bits: cl_map_flags) -> Self {
        #[cfg(feature = "cl1_2")]
        if (bits & CL_MAP_WRITE_INVALIDATE_REGION) != 0 {
            return Self::WriteInvalidate
        }

        Self::Access(MemAccess::from_bits_map(bits))
    }

    #[inline(always)]
    pub const fn to_bits (self) -> cl_map_flags {
        match self {
            Self::Access(access) => access.to_bits_map(),
            #[cfg(feature = "cl1_2")]
            Self::WriteInvalidate => CL_MAP_WRITE_INVALIDATE_REGION
        }
    }
}

impl Default for MapFlags {
    #[inline(always)]
    fn default() -> Self {
        Self::Access(MemAccess::default())
    }
}