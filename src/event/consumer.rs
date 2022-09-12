use std::{marker::PhantomData, panic::{UnwindSafe, catch_unwind}, any::Any};
use blaze_proc::docfg;
use crate::prelude::Result;

/// A trait that represents the consumer of an [`Event`](super::Event)
pub trait Consumer<'a>: 'a {
    type Output;
    fn consume (self) -> Result<Self::Output>;
}

impl<'a, T: 'a> Consumer<'a> for Result<T> {
    type Output = T;

    #[inline(always)]
    fn consume (self) -> Result<T> {
        self
    }
}

impl<'a, T, F: 'a + FnOnce() -> Result<T>> Consumer<'a> for F {
    type Output = T;

    #[inline(always)]
    fn consume (self) -> Result<T> {
        (self)()
    }
}

/// **No**-**op**eration trait consumer.
#[repr(transparent)]
pub struct Noop<'a> (PhantomData<&'a ()>);

impl Noop<'_> {
    #[inline(always)]
    pub const fn new () -> Self { Self(PhantomData) } 
}

impl<'a> Consumer<'a> for Noop<'a> {
    type Output = ();

    #[inline(always)]
    fn consume (self) -> Result<()> {
        Ok(())
    }
}

/// Consumer for [`map`](super::Event::map) event.
pub struct Map<T, C, F> (pub(crate) C, pub(crate) F, PhantomData<T>);

impl<'a, 'b, T, U, C: Consumer<'a, Output = T>, F: 'b + FnOnce(T) -> U> Map<T, C, F> where 'a: 'b {
    #[inline(always)]
    pub const fn new (consumer: C, f: F) -> Self { Self(consumer, f, PhantomData) } 
}

impl<'a: 'b, 'b, T: 'b, U, C: Consumer<'a, Output = T>, F: 'b + FnOnce(T) -> U> Consumer<'b> for Map<T, C, F> {
    type Output = U;

    #[inline(always)]
    fn consume (self) -> Result<U> {
        let v = self.0.consume()?;
        return Ok((self.1)(v))
    }
}

/// Consumer for [`try_map`](super::Event::try_map) event.
pub struct TryMap<T, C, F> (pub(crate) C, pub(crate) F, PhantomData<T>);

impl<'a, 'b, T, U, C: Consumer<'a, Output = T>, F: 'b + FnOnce(T) -> Result<U>> TryMap<T, C, F> where 'a: 'b {
    #[inline(always)]
    pub const fn new (consumer: C, f: F) -> Self { Self(consumer, f, PhantomData) } 
}

impl<'a: 'b, 'b, T: 'b, U, C: Consumer<'a, Output = T>, F: 'b + FnOnce(T) -> Result<U>> Consumer<'b> for TryMap<T, C, F> {
    type Output = U;

    #[inline(always)]
    fn consume (self) -> Result<U> {
        let v = self.0.consume()?;
        return (self.1)(v)
    }
}

/// Consumer for [`catch_unwind`](super::Event::catch_unwind) event.
#[repr(transparent)]
pub struct CatchUnwind<C: UnwindSafe> (pub(super) C);

impl<'a, C: Consumer<'a> + UnwindSafe> Consumer<'a> for CatchUnwind<C> {
    type Output = ::core::result::Result<C::Output, Box<dyn Any + Send>>;

    #[inline(always)]
    fn consume (self) -> Result<Self::Output> {
        return match catch_unwind(|| self.0.consume()) {
            Ok(Ok(x)) => Ok(Ok(x)),
            Ok(Err(e)) => Err(e),
            Err(e) => Ok(Err(e))
        }
    }
} 

/// Consumer for [`flatten`](super::Event::flatten) event.
#[repr(transparent)]
pub struct Flatten<C> (pub(super) C);

impl<'a, T, C: Consumer<'a, Output = Result<T>>> Consumer<'a> for Flatten<C> {
    type Output = T;

    #[inline(always)]
    fn consume (self) -> Result<T> {
        self.0.consume().flatten()
    }
}

/// Consumer for [`inspect`](super::Event::inspect) event.
pub struct Inspect<C, F> (pub(super) C, pub(super) F);

impl<'a, C: Consumer<'a>, F: 'a + FnOnce(&C::Output)> Consumer<'a> for Inspect<C, F> {
    type Output = C::Output;

    #[inline(always)]
    fn consume (self) -> Result<C::Output> {
        let v = self.0.consume()?;
        (self.1)(&v);
        return Ok(v)
    }
}

/// Consumer for [`join_all`](super::Event::join_all) event.
#[docfg(feature = "cl1_1")]
pub struct JoinAll<C> (pub(super) Vec<C>);

#[cfg(feature = "cl1_1")]
impl<'a, C: Consumer<'a>> Consumer<'a> for JoinAll<C> {
    type Output = Vec<C::Output>;

    #[inline]
    fn consume (self) -> Result<Vec<C::Output>> {
        self.0.into_iter().map(Consumer::consume).try_collect()
    }
}