use std::mem::MaybeUninit;
use blaze_proc::blaze;

#[blaze(Arith<T: Sync>)]
#[link = "hello"]
pub extern "C" {
    fn test (n: u32, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
}