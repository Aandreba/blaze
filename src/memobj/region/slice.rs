use std::{num::NonZeroUsize, ops::{RangeBounds, Bound}};
use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range2D {
    pub offset_x: usize,
    pub offset_y: usize,
    pub region_x: NonZeroUsize,
    pub region_y: NonZeroUsize
}

impl Range2D {
    #[inline(always)]
    pub const fn new (offset_x: usize, offset_y: usize, region_x: NonZeroUsize, region_y: NonZeroUsize) -> Self {
        Self { offset_x, offset_y, region_x, region_y }
    }

    #[inline(always)]
    pub fn try_new (offset_x: usize, offset_y: usize, region_x: usize, region_y: usize) -> Result<Self> {
        macro_rules! _tri_ {
            ($e:expr, $desc:expr) => {
                $e.ok_or_else(|| Error::new(ErrorKind::InvalidBufferSize, $desc))?
            };
        }

        let region_x = _tri_!(NonZeroUsize::new(region_x), "region x is zero");
        let region_y = _tri_!(NonZeroUsize::new(region_y), "region y is zero");
        Ok(Self::new(offset_x, offset_y, region_x, region_y))
    }

    pub fn from_range<X: RangeBounds<usize>, Y: RangeBounds<usize>> (x: X, y: Y, max_x: usize, max_y: usize) -> Result<Self> {
        macro_rules! _tri_ {
            ($e:expr, $desc:expr) => {
                $e.ok_or_else(|| Error::new(ErrorKind::InvalidBufferSize, $desc))?
            };
        }

        let offset_x = match x.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => _tri_!(x.checked_add(1), "overflow calculating range offset x"),
            Bound::Unbounded => 0
        };

        let offset_y = match y.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => _tri_!(x.checked_add(1), "overflow calculating range offset y"),
            Bound::Unbounded => 0
        };

        let region_x = _tri_!(match x.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => _tri_!(x.checked_add(1), "overflow calculating range region x"),
            Bound::Unbounded => max_x
        }.checked_sub(offset_x), "overflow calculating range region x");

        let region_y = _tri_!(match y.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => _tri_!(x.checked_add(1), "overflow calculating range region x"),
            Bound::Unbounded => max_y
        }.checked_sub(offset_y), "overflow calculating range region y");

        let region_x = _tri_!(NonZeroUsize::new(region_x), "region x is zero");
        let region_y = _tri_!(NonZeroUsize::new(region_y), "region y is zero");

        Ok(Self { offset_x, offset_y, region_x, region_y })
    }

    #[inline(always)]
    pub fn width (&self) -> usize {
        self.region_x.get()
    }

    #[inline(always)]
    pub fn height (&self) -> usize {
        self.region_y.get()
    }

    #[inline(always)]
    pub fn size (&self) -> Option<NonZeroUsize> {
        self.region_x.checked_mul(self.region_y)
    }

    #[inline]
    pub fn raw_parts (&self) -> [[usize;3];2] {
        let offset = [self.offset_x, self.offset_y, 0];
        let region = [self.region_x.get(), self.region_y.get(), 1];
        [offset, region]
    }

    #[inline]
    pub fn raw_parts_buffer<T> (&self) -> [[usize;3];2] {
        let offset = [self.offset_x * core::mem::size_of::<T>(), self.offset_y, 0];
        let region = [self.region_x.get() * core::mem::size_of::<T>(), self.region_y.get(), 1];
        [offset, region]
    }
}

pub trait IntoRange2D {
    fn into_range (self, max_x: usize, max_y: usize) -> Result<Range2D>;
}

impl IntoRange2D for Range2D {
    #[inline(always)]
    fn into_range (self, _max_x: usize, _max_y: usize) -> Result<Range2D> {
        Ok(self)
    }
}

impl<X: RangeBounds<usize>, Y: RangeBounds<usize>> IntoRange2D for (X, Y) {
    #[inline(always)]
    fn into_range (self, max_x: usize, max_y: usize) -> Result<Range2D> {
        Range2D::from_range(self.0, self.1, max_x, max_y)
    }
}

impl IntoRange2D for [[usize;2];2] {
    fn into_range (self, _max_x: usize, _max_y: usize) -> Result<Range2D> {
        Range2D::try_new(self[0][0], self[0][1], self[1][0], self[1][1])
    }
}