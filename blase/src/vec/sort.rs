use std::mem::MaybeUninit;

use blaze_rs::{prelude::{Result, WaitList, Buffer, MemAccess, Event, RawEvent, EventExt}, event::various::Join};
use crate::{Real};
use super::EucVec;

pub struct Sort<T: Real> {
    result: Buffer<MaybeUninit<T>>,
    join: Join<RawEvent>
}

impl<T: Real> Event for Sort<T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn as_raw (&self) -> &RawEvent {
        self.join.as_raw()
    }

    #[inline(always)]
    fn consume (self, err: Option<blaze_rs::prelude::Error>) -> Result<Self::Output> {
        let _ = self.join.consume(err)?;
        unsafe {
            Ok(EucVec::from_buffer(self.result.assume_init()))
        }
    }
}

impl<T: Real> EucVec<T> {
    // https://github.com/Gram21/GPUSorting/blob/master/Code/CSortTask.cpp
    pub fn sort (&self, desc: bool, wait: impl Into<WaitList>) -> Result<Sort<T>> {
        let local_work_size = 256;
        let wait : WaitList = wait.into();

        let size = self.len()?;
        let padded_size = size.next_power_of_two();

        let local_wgs = [local_work_size];
        let global_wgs = [get_global_work_size(padded_size / 2, local_work_size)];
        let limit = 2 * local_work_size;

        let mut pong = Buffer::<T>::new_uninit(padded_size, MemAccess::default(), false)?;
        
        let start = unsafe {
            T::vec_program().sort_start(
                desc, self, &mut pong,
                global_wgs, local_wgs,
                wait.clone()
            )?
        };

        let mut events = vec![start.to_raw()];
        let mut blocksize = limit;

        while blocksize <= padded_size {
            let mut stride = blocksize / 2;
            while stride > 0 {
                events.push(match stride >= limit {
                    true => unsafe {
                        T::vec_program().sort_global(
                            desc, &mut pong, padded_size, blocksize, stride,
                            global_wgs, local_wgs,
                            wait.clone()
                        )?.to_raw()
                    },

                    false => unsafe {
                        T::vec_program().sort_local(
                            desc, &mut pong, padded_size, blocksize, stride,
                            global_wgs, local_wgs,
                            wait.clone()
                        )?.to_raw()
                    }
                });

                stride >>= 1;
            }

            blocksize <<= 1;
        }

        Ok(Sort {
            result: pong,
            join: RawEvent::join(events)?
        })
    }
}

#[inline(always)]
const fn get_global_work_size (data_elem_count: usize, local_work_size: usize) -> usize {
    match data_elem_count % local_work_size {
        0 => data_elem_count,
        r => data_elem_count + local_work_size - r,
    }
}

#[cfg(test)]
mod tests {
    use blaze_rs::prelude::*;
    use crate::{vec::EucVec, random::Random};

    #[test]
    fn test () -> Result<()> {
        let mut rng = Random::new(None)?;

        let buf = rng.next_u32(5, 0..=5, true, false)?;
        println!("{buf:?}");
        let sorted = EucVec::from_buffer(buf).sort(false, EMPTY)?.wait()?;
        println!("{sorted:?}");

        assert!(sorted.map(.., EMPTY)?.wait()?.is_sorted());
        Ok(())
    }
}