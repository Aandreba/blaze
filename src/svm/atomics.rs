use crate::context::{Context, Global};
use super::{SvmBox, Svm};

#[repr(transparent)]
pub struct SvmAtomicU32<C: Context = Global> (SvmBox<[u32], C>);

impl<C: Context> SvmAtomicU32<C> {
    pub fn new_in <const N: usize> (v: [u32; N], ctx: C) {
        let boxed = SvmBox::new_in(x, Svm::ne);
    }

    #[inline(always)]
    pub const unsafe fn from_box_in (inner: SvmBox<[u32], C>) -> Self {
        Self(inner)
    }
}
