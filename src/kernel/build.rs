use super::Kernel;
use crate::core::*;

pub struct Build<'a> {
    parent: &'a Kernel
}

impl<'a> Build<'a> {
    #[inline(always)]
    pub fn new (parent: &'a Kernel) -> Result<Self> {
        let args = parent.num_args()? as usize;
        todo!()
    }
}