use std::mem::MaybeUninit;
use blaze_proc::NumOps;

#[derive(Clone, NumOps)]
#[repr(transparent)]
pub struct Num<T: Copy> (T);