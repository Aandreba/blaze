use std::{ops::{RangeBounds, Bound}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSlice {
    pub offset: [usize; 3],
    pub region: [usize; 3]
}

impl ImageSlice {
    #[inline(always)]
    pub fn new_2d (offset: [usize; 2], region: [usize; 2]) -> Self {
        let mut new_offset = [0; 3];
        let mut new_region = [1; 3];

        unsafe {
            core::ptr::copy_nonoverlapping(offset.as_ptr(), new_offset.as_mut_ptr(), 2);
            core::ptr::copy_nonoverlapping(region.as_ptr(), new_region.as_mut_ptr(), 2);
        }

        Self::new_3d(new_offset, new_region)
    }

    #[inline(always)]
    pub const fn new_3d (offset: [usize; 3], region: [usize; 3]) -> Self {
        Self { offset, region }
    }

    pub fn from_range_2d<X: RangeBounds<usize>, Y: RangeBounds<usize>> (x: X, y: Y, max_x: usize, max_y: usize) -> Self {
        let offset_x = match x.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x.checked_add(1).unwrap(),
            Bound::Unbounded => 0
        };

        let offset_y = match y.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x.checked_add(1).unwrap(),
            Bound::Unbounded => 0
        };

        let region_x = match x.end_bound() {
            Bound::Included(x) => x.checked_add(1).unwrap(),
            Bound::Excluded(x) => *x,
            Bound::Unbounded => max_x
        } - offset_x;

        let region_y = match y.end_bound() {
            Bound::Included(x) => x.checked_add(1).unwrap(),
            Bound::Excluded(x) => *x,
            Bound::Unbounded => max_y
        } - offset_y;

        Self::new_3d([offset_x, offset_y, 0], [region_x, region_y, 1])
    }

    #[inline(always)]
    pub fn width (&self) -> usize {
        self.region[0]
    }

    #[inline(always)]
    pub fn height (&self) -> usize {
        self.region[1]
    }

    #[inline(always)]
    pub fn size (&self) -> usize {
        self.region.into_iter().product()
    }

    #[inline(always)]
    pub fn raw_parts (&self) -> (*const usize, *const usize) {
        (self.offset.as_ptr(), self.region.as_ptr())
    }
}

impl From<[[usize;2];2]> for ImageSlice {
    #[inline(always)]
    fn from(x: [[usize;2];2]) -> Self {
        let [offset, region] = x;
        Self::new_2d(offset, region)
    }
}

impl From<[[usize;3];2]> for ImageSlice {
    #[inline(always)]
    fn from(x: [[usize;3];2]) -> Self {
        let [offset, region] = x;
        Self::new_3d(offset, region)
    }
}

pub trait IntoSlice<const N: usize> {
    fn into_slice (self, dims: [usize;N]) -> ImageSlice;
}

impl<const N: usize> IntoSlice<N> for ImageSlice {
    #[inline(always)]
    fn into_slice (self, _dims: [usize;N]) -> ImageSlice {
        self
    }
}

impl<const N: usize> IntoSlice<N> for [[usize;2];2] {
    #[inline(always)]
    fn into_slice (self, _dims: [usize;N]) -> ImageSlice {
        self.into()
    }
}

impl<const N: usize> IntoSlice<N> for [[usize;3];2] {
    #[inline(always)]
    fn into_slice (self, _dims: [usize;N]) -> ImageSlice {
        self.into()
    }
}

impl<X: RangeBounds<usize>, Y: RangeBounds<usize>> IntoSlice<2> for (X, Y) {
    #[inline(always)]
    fn into_slice (self, dims: [usize;2]) -> ImageSlice {
        let [width, height] = dims;
        ImageSlice::from_range_2d(self.0, self.1, width, height)
    }
}