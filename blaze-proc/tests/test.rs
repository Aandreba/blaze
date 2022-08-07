use blaze_proc::blaze;

#[blaze(Arith<T>)]
#[link = "hello"]
pub extern "C" {
    fn test (n: u32, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
}