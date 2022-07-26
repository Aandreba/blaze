use std::mem::MaybeUninit;
use rscl_proc::NumOps;

#[derive(Clone, NumOps)]
#[repr(transparent)]
pub struct Num<T: Copy> (T);