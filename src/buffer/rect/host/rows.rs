use std::alloc::Allocator;
use super::Rect2D;

pub struct Rows<'a, T, A: Allocator> {
    pub(super) inner: &'a Rect2D<T, A>,
    pub(super) idx: usize
}

impl<'a, T, A: Allocator> Iterator for Rows<'a, T, A> {
    type Item = &'a [T];

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.inner.get_row(self.idx);
        if let Some(idx) = self.idx.checked_add(1) {
            self.idx = idx;
        }

        v
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, T, A: Allocator> ExactSizeIterator for Rows<'a, T, A> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.rows.get() - self.idx
    }
}