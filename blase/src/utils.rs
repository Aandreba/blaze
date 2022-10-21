use std::{marker::PhantomData};
use blaze_rs::{event::Consumer};

#[repr(transparent)]
pub struct ValueConsumer<T, P: ?Sized> {
    inner: T,
    phtm: PhantomData<P>
}

impl<T, P: ?Sized> ValueConsumer<T, P> {
    #[inline(always)]
    pub const fn new (inner: T, phtm: PhantomData<P>) -> Self {
        Self { inner, phtm }
    }
}

impl<T, P: ?Sized> Consumer for ValueConsumer<T, P> {
    type Output = T;

    #[inline(always)]
    unsafe fn consume (self) -> blaze_rs::prelude::Result<Self::Output> {
        Ok(self.inner)
    }
}

#[allow(unused)]
#[inline(always)]
pub(crate) const unsafe fn change_lifetime<'a, 'b, T: ?Sized> (v: &'a T) -> &'b T {
    core::mem::transmute(v)
}

#[inline(always)]
pub(crate) unsafe fn change_lifetime_mut<'a, 'b, T: ?Sized> (v: &'a mut T) -> &'b mut T {
    core::mem::transmute(v)
}