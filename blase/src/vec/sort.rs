use blaze_rs::prelude::{Result, WaitList, Event};
use crate::{Real, work_group_size};
use super::EucVec;

impl<T: Real> EucVec<T> {
    pub fn sort (&mut self, wait: impl Into<WaitList>) -> Result<()> {
        let mut this = self;
        
        let len = this.len()?;
        let wgs = work_group_size(len);
        let init_block_size = len / wgs;

        let evt = unsafe {
            let evt = T::vec_program().block_sort(len, this, [wgs], None, wait)?;
            let raw = evt.to_raw();
            this = evt.consume(None)?;
            raw
        };

        let evt = unsafe {
            T::vec_program().merge_blocks(len, init_block_size, this, [wgs], None, WaitList::from_event(evt))?
        };

        evt.wait_by_ref()
    }
}

#[cfg(test)]
mod tests {
    use blaze_rs::prelude::*;
    use crate::vec::EucVec;

    //#[global_context]
    //static CTX : SimpleContext = SimpleContext::default();

    #[test]
    fn test () -> Result<()> {
        let mut buf = EucVec::new(&[1, 8, 3, 9, 4, 5, 7], false)?;
        buf.sort(EMPTY)?;

        println!("{buf:?}");
        Ok(())
    }
}