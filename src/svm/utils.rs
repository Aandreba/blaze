use std::collections::VecDeque;
use crate::context::Global;
use super::{Svm};
use sealed::Sealed;

mod sealed {
    pub trait Sealed {}
}

const ALLOC : Svm = Svm::new();

pub type SvmBox<T, C = Global> = Box<T, Svm<C>>;
pub type SvmVec<T, C = Global> = Vec<T, Svm<C>>;
pub type SvmVecDeque<T, C = Global> = VecDeque<T, Svm<C>>;

impl<T: ?Sized> Sealed for SvmBox<T> {}
impl<T> Sealed for SvmVec<T> {}
impl<T> Sealed for SvmVecDeque<T> {}

// BOX
pub trait SvmBoxExt<T: ?Sized>: Sealed {
    fn new (v: T) -> Self where T: Sized;
}

impl<T: ?Sized> SvmBoxExt<T> for SvmBox<T> {
    #[inline(always)]
    fn new (v: T) -> Self where T: Sized {
        Self::new_in(v, ALLOC)
    }
} 

// VEC
pub trait SvmVecExt<T>: Sealed + Sized {
    fn with_capacity (cap: usize) -> Self;

    #[inline(always)]
    fn new () -> Self {
        Self::with_capacity(0)
    }
}

impl<T> SvmVecExt<T> for SvmVec<T> {
    #[inline(always)]
    fn with_capacity (cap: usize) -> Self {
        Self::with_capacity_in(cap, ALLOC)
    }
}

// VEC DEQUEUE
pub trait SvmVecDequeExt<T>: Sealed + Sized {
    fn with_capacity (cap: usize) -> Self;

    #[inline(always)]
    fn new () -> Self {
        const INITIAL_CAPACITY: usize = 7; // 2^3 - 1
        Self::with_capacity(INITIAL_CAPACITY)
    }
}

impl<T> SvmVecDequeExt<T> for SvmVecDeque<T> {
    #[inline(always)]
    fn with_capacity (cap: usize) -> Self {
        Self::with_capacity_in(cap, ALLOC)
    }
}