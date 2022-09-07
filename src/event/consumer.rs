use std::{marker::PhantomData};
use crate::prelude::Result;

/// A trait that represents the consumer of an [`Event`]
pub trait Consumer<'a, T>: 'a {
    fn consume (self) -> Result<T>;
}

impl<'a, T: 'a> Consumer<'a, T> for Result<T> {
    #[inline(always)]
    fn consume (self) -> Result<T> {
        self
    }
}

impl<'a, T, F: 'a + FnOnce() -> Result<T>> Consumer<'a, T> for F {
    #[inline(always)]
    fn consume (self) -> Result<T> {
        (self)()
    }
}

/// **No**-**op**eration trait consumer
pub struct Noop<'a> (PhantomData<&'a ()>);

impl Noop<'_> {
    #[inline(always)]
    pub const fn new () -> Self { Self(PhantomData) } 
}

impl<'a> Consumer<'a, ()> for Noop<'a> {
    #[inline(always)]
    fn consume (self) -> Result<()> {
        Ok(())
    }
}