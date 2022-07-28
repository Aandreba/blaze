use std::ops::{RangeBounds, Bound};
use opencl_sys::*;
use crate::prelude::*;
use super::RawMemObject;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MemObjectType {
    Buffer = CL_MEM_OBJECT_BUFFER,
    Pipe = CL_MEM_OBJECT_PIPE,
    Image1D = CL_MEM_OBJECT_IMAGE1D,
    Image2D = CL_MEM_OBJECT_IMAGE2D,
    Image3D = CL_MEM_OBJECT_IMAGE3D,
    Image1DArray = CL_MEM_OBJECT_IMAGE1D_ARRAY,
    Image2DArray = CL_MEM_OBJECT_IMAGE2D_ARRAY,
    Image1DBuffer = CL_MEM_OBJECT_IMAGE1D_BUFFER,
}

impl Into<u32> for MemObjectType {
    #[inline(always)]
    fn into(self) -> u32 {
        self as u32
    }
}

#[allow(unused)]
#[inline]
pub(crate) fn offset_cb_plain (buffer: &RawMemObject, range: impl RangeBounds<usize>) -> Result<(usize, usize)> {
    let start = match range.start_bound() {
        Bound::Excluded(x) => x.checked_add(1).unwrap(),
        Bound::Included(x) => *x,
        Bound::Unbounded => 0
    };

    let end = match range.end_bound() {
        Bound::Excluded(x) => *x,
        Bound::Included(x) => x.checked_add(1).unwrap(),
        Bound::Unbounded => buffer.size()?
    };

    let len = end - start;
    Ok((start, len))
}
