#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ordering {
    RowMajor,
    ColMajor
}

impl Default for Ordering {
    #[inline(always)]
    fn default() -> Self {
        Self::ColMajor
    }
}

