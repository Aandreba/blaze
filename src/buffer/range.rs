use std::ops::*;
use crate::prelude::*;
use super::{RawBuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferRange {
    pub offset: usize,
    pub cb: usize
}

impl BufferRange {
    #[inline(always)]
    pub const fn new (offset: usize, cb: usize) -> Self {
        Self { offset, cb }
    }

    #[inline]
    pub fn from_parts<T> (offset: usize, size: usize) -> Result<Self> {
        let offset = match offset.checked_mul(core::mem::size_of::<T>()) {
            Some(x) => x,
            None => return Err(Error::new(ErrorKind::InvalidBufferSize, "overflow calculating range offset"))
        };

        let size = match size.checked_mul(core::mem::size_of::<T>()) {
            Some(x) => x,
            None => return Err(Error::new(ErrorKind::InvalidBufferSize, "overflow calculating range size"))
        };

        Ok(Self::new(offset, size))
    }

    #[inline(always)]
    pub fn from_byte_range<R: RangeBounds<usize>> (range: R, max_size: usize) -> Result<Self> {
        Self::from_range::<u8, R>(range, max_size)
    }

    #[inline]
    pub fn from_range<T, R: RangeBounds<usize>> (range: R, max_size: usize) -> Result<Self> {
        macro_rules! _tri_ {
            ($e:expr, $desc:expr) => {
                $e.ok_or_else(|| Error::new(ErrorKind::InvalidBufferSize, $desc))?
            };
        }

        let offset = match range.start_bound() {
            Bound::Excluded(x) => _tri_!(_tri_!(x.checked_add(1), "overflow calculating range offset").checked_mul(core::mem::size_of::<T>()), "overflow calculating range offset"),
            Bound::Included(x) => _tri_!(x.checked_mul(core::mem::size_of::<T>()), "overflow calculating range offset"),
            Bound::Unbounded => 0
        };

        let size = match range.end_bound() {
            Bound::Excluded(x) => _tri_!(x.checked_mul(core::mem::size_of::<T>()), "overflow calculating range size"),
            Bound::Included(x) => _tri_!(_tri_!(x.checked_add(1), "overflow calculating range size").checked_mul(core::mem::size_of::<T>()), "overflow calculating range offset"),
            Bound::Unbounded => max_size
        } - offset;

        return Ok(Self::new(offset, size))
    }
}

pub trait IntoRange {
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange>;
}

impl IntoRange for BufferRange {
    #[inline(always)]
    fn into_range<T> (self, _buf: &RawBuffer) -> Result<BufferRange> {
        Ok(self)
    }
}

impl IntoRange for Range<usize> {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        BufferRange::from_range::<T, Self>(self, buf.size()?)
    }
}

impl IntoRange for RangeFrom<usize> {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        BufferRange::from_range::<T, Self>(self, buf.size()?)
    }
}

impl IntoRange for RangeFull {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        BufferRange::from_range::<T, Self>(self, buf.size()?)
    }
}

impl IntoRange for RangeInclusive<usize> {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        BufferRange::from_range::<T, Self>(self, buf.size()?)
    }
}

impl IntoRange for RangeTo<usize> {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        BufferRange::from_range::<T, Self>(self, buf.size()?)
    }
}

impl IntoRange for RangeToInclusive<usize> {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        BufferRange::from_range::<T, Self>(self, buf.size()?)
    }
}

impl IntoRange for [usize; 2] {
    #[inline(always)]
    fn into_range<T> (self, _buf: &RawBuffer) -> Result<BufferRange> {
        BufferRange::from_parts::<T>(self[0], self[1])
    }
}

impl IntoRange for (usize, usize) {
    #[inline(always)]
    fn into_range<T> (self, _buf: &RawBuffer) -> Result<BufferRange> {
        BufferRange::from_parts::<T>(self.0, self.1)
    }
}