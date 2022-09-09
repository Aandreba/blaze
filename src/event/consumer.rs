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
#[repr(transparent)]
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

pub struct Map<T, C, F> (pub(crate) C, pub(crate) F, PhantomData<T>);

impl<'a, 'b, T, U, C: Consumer<'a, T>, F: 'b + FnOnce(T) -> U> Map<T, C, F> where 'a: 'b {
    #[inline(always)]
    pub const fn new (consumer: C, f: F) -> Self { Self(consumer, f, PhantomData) } 
}

impl<'a: 'b, 'b, T: 'b, U, C: Consumer<'a, T>, F: 'b + FnOnce(T) -> U> Consumer<'b, U> for Map<T, C, F> {
    #[inline(always)]
    fn consume (self) -> Result<U> {
        let v = self.0.consume()?;
        return Ok((self.1)(v))
    }
}