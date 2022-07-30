use std::{ptr::NonNull};
use opencl_sys::*;
use crate::prelude::MemAccess;
use super::AsMem;

/// Guard for mapped memory object region
pub struct MapGuard<T, M: AsMem> {
    ptr: NonNull<[T]>,
    mem: M
}

impl<T, M: AsMem> Drop for MapGuard<T, M> {
    fn drop(&mut self) {
        todo!()
    }
}

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