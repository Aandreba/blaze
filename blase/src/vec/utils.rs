use std::{mem::MaybeUninit, marker::PhantomData};
use blaze_rs::event::{Consumer, IncompleteConsumer};
use super::EucVec;

pub struct Binary<'a, T: Copy> {
    inner: EucVec<MaybeUninit<T>>,
    phtm: PhantomData<(&'a EucVec<T>, &'a EucVec<T>)>
}

impl<'a, T: Copy> Binary<'a, T> {
    #[inline(always)]
    pub(super) const fn new (inner: EucVec<MaybeUninit<T>>) -> Self {
        Self { inner, phtm: PhantomData }
    }
}

impl<T: Copy> Consumer for Binary<'_, T> {
    type Output = EucVec<T>;

    #[inline(always)]
    fn consume (self) -> blaze_rs::prelude::Result<Self::Output> {
        unsafe { Ok(self.inner.assume_init()) }
    }
}

impl<T: Copy> IncompleteConsumer for Binary<'_, T> {
    type Incomplete = EucVec<MaybeUninit<T>>;

    #[inline(always)]
    fn consume_incomplete (self) -> blaze_rs::prelude::Result<Self::Incomplete> {
        return Ok(self.inner)
    }
}