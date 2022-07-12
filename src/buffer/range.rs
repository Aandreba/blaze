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
    pub const fn from_parts<T> (offset: usize, size: usize) -> Option<Self> {
        let offset = match offset.checked_mul(core::mem::size_of::<T>()) {
            Some(x) => x,
            None => return None
        };

        let size = match size.checked_mul(core::mem::size_of::<T>()) {
            Some(x) => x,
            None => return None
        };

        Some(Self::new(offset, size))
    }

    #[inline(always)]
    pub fn from_byte_range (range: impl RangeBounds<usize>, max_size: usize) -> Option<Self> {
        let offset = match range.start_bound() {
            Bound::Excluded(x) => x.checked_add(1)?,
            Bound::Included(x) => *x,
            Bound::Unbounded => 0
        };

        let size = match range.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => x.checked_add(1)?,
            Bound::Unbounded => max_size
        } - offset;

        Some(Self::new(offset, size))
    }

    #[inline(always)]
    pub fn from_range<T, R: RangeBounds<usize>> (range: R, max_size: usize) -> Option<Self> {
        let offset = match range.start_bound() {
            Bound::Excluded(x) => x.checked_add(1)?.checked_mul(core::mem::size_of::<T>())?,
            Bound::Included(x) => x.checked_mul(core::mem::size_of::<T>())?,
            Bound::Unbounded => 0
        };

        let size = match range.end_bound() {
            Bound::Excluded(x) => x.checked_mul(core::mem::size_of::<T>())?,
            Bound::Included(x) => x.checked_add(1)?.checked_mul(core::mem::size_of::<T>())?,
            Bound::Unbounded => max_size
        } - offset;

        Some(Self::new(offset, size))
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
        Ok(BufferRange::from_range::<T, Self>(self, buf.size()?).unwrap())
    }
}

impl IntoRange for RangeFrom<usize> {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        Ok(BufferRange::from_range::<T, Self>(self, buf.size()?).unwrap())
    }
}

impl IntoRange for RangeFull {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        Ok(BufferRange::from_range::<T, Self>(self, buf.size()?).unwrap())
    }
}

impl IntoRange for RangeInclusive<usize> {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        Ok(BufferRange::from_range::<T, Self>(self, buf.size()?).unwrap())
    }
}

impl IntoRange for RangeTo<usize> {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        Ok(BufferRange::from_range::<T, Self>(self, buf.size()?).unwrap())
    }
}

impl IntoRange for RangeToInclusive<usize> {
    #[inline(always)]
    fn into_range<T> (self, buf: &RawBuffer) -> Result<BufferRange> {
        Ok(BufferRange::from_range::<T, Self>(self, buf.size()?).unwrap())
    }
}

impl IntoRange for [usize; 2] {
    #[inline(always)]
    fn into_range<T> (self, _buf: &RawBuffer) -> Result<BufferRange> {
        Ok(BufferRange::from_parts::<T>(self[0], self[1]).unwrap())
    }
}

impl IntoRange for (usize, usize) {
    #[inline(always)]
    fn into_range<T> (self, _buf: &RawBuffer) -> Result<BufferRange> {
        Ok(BufferRange::from_parts::<T>(self.0, self.1).unwrap())
    }
}