use std::{num::NonZeroUsize, ops::{RangeBounds, Bound}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slice2D {
    pub offset_x: usize,
    pub offset_y: usize,
    pub region_x: NonZeroUsize,
    pub region_y: NonZeroUsize
}

impl Slice2D {
    #[inline(always)]
    pub const fn new (offset_x: usize, offset_y: usize, region_x: NonZeroUsize, region_y: NonZeroUsize) -> Self {
        Self { offset_x, offset_y, region_x, region_y }
    }

    #[inline(always)]
    pub fn try_new (offset_x: usize, offset_y: usize, region_x: usize, region_y: usize) -> Option<Self> {
        let region_x = NonZeroUsize::new(region_x)?;
        let region_y = NonZeroUsize::new(region_y)?;
        Some(Self::new(offset_x, offset_y, region_x, region_y))
    }

    pub fn from_range<X: RangeBounds<usize>, Y: RangeBounds<usize>> (x: X, y: Y, max_x: usize, max_y: usize) -> Option<Self> {
        let offset_x = match x.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x.checked_add(1)?,
            Bound::Unbounded => 0
        };

        let offset_y = match y.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x.checked_add(1)?,
            Bound::Unbounded => 0
        };

        let region_x = match x.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => x.checked_add(1)?,
            Bound::Unbounded => max_x
        }.checked_sub(offset_x)?;

        let region_y = match y.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => x.checked_add(1)?,
            Bound::Unbounded => max_y
        }.checked_sub(offset_y)?;

        let region_x = NonZeroUsize::new(region_x)?;
        let region_y = NonZeroUsize::new(region_y)?;

        Some(Self { offset_x, offset_y, region_x, region_y })
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
        let offset = [self.offset_x, self.offset_y, 0];
        let region = [self.region_x.get() * core::mem::size_of::<T>(), self.region_y.get(), 1];
        [offset, region]
    }
}

pub trait IntoSlice2D {
    fn into_slice (self, max_x: usize, max_y: usize) -> Option<Slice2D>;
}

impl IntoSlice2D for Slice2D {
    #[inline(always)]
    fn into_slice (self, _max_x: usize, _max_y: usize) -> Option<Slice2D> {
        Some(self)
    }
}

impl<X: RangeBounds<usize>, Y: RangeBounds<usize>> IntoSlice2D for (X, Y) {
    #[inline(always)]
    fn into_slice (self, max_x: usize, max_y: usize) -> Option<Slice2D> {
        Slice2D::from_range(self.0, self.1, max_x, max_y)
    }
}

impl IntoSlice2D for [[usize;2];2] {
    fn into_slice (self, _max_x: usize, _max_y: usize) -> Option<Slice2D> {
        Slice2D::try_new(self[0][0], self[0][1], self[1][0], self[0][0])
    }
}