use std::mem::MaybeUninit;
use blaze_proc::blaze;

#[blaze(Arith)]
#[link = "hello"]
pub extern "C" {
    fn test (n: u32, lhs: *const f32, rhs: *const f32, out: *mut MaybeUninit<f32>);
}