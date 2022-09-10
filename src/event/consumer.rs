use std::{marker::PhantomData, panic::{UnwindSafe, catch_unwind}, any::Any};
use blaze_proc::docfg;
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

/// **No**-**op**eration trait consumer.
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

/// Consumer for [`map`](super::Event::map) event.
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

/// Consumer for [`try_map`](super::Event::try_map) event.
pub struct TryMap<T, C, F> (pub(crate) C, pub(crate) F, PhantomData<T>);

impl<'a, 'b, T, U, C: Consumer<'a, T>, F: 'b + FnOnce(T) -> Result<U>> TryMap<T, C, F> where 'a: 'b {
    #[inline(always)]
    pub const fn new (consumer: C, f: F) -> Self { Self(consumer, f, PhantomData) } 
}

impl<'a: 'b, 'b, T: 'b, U, C: Consumer<'a, T>, F: 'b + FnOnce(T) -> Result<U>> Consumer<'b, U> for TryMap<T, C, F> {
    #[inline(always)]
    fn consume (self) -> Result<U> {
        let v = self.0.consume()?;
        return (self.1)(v)
    }
}

/// Consumer for [`catch_unwind`](super::Event::catch_unwind) event.
#[repr(transparent)]
pub struct CatchUnwind<C: UnwindSafe> (pub(super) C);

impl<'a, T, C: Consumer<'a, T> + UnwindSafe> Consumer<'a, ::core::result::Result<T, Box<dyn Any + Send>>> for CatchUnwind<C> {
    #[inline(always)]
    fn consume (self) -> Result<::core::result::Result<T, Box<dyn Any + Send>>> {
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

impl<'a, T, C: Consumer<'a, Result<T>>> Consumer<'a, T> for Flatten<C> {
    #[inline(always)]
    fn consume (self) -> Result<T> {
        self.0.consume().flatten()
    }
}

/// Consumer for [`inspect`](super::Event::inspect) event.
pub struct Inspect<C, F> (pub(super) C, pub(super) F);

impl<'a, 'b, T, C: Consumer<'a, T>, F: 'b + FnOnce(&T)> Consumer<'b, T> for Inspect<C, F> where 'a: 'b {
    #[inline(always)]
    fn consume (self) -> Result<T> {
        let v = self.0.consume()?;
        (self.1)(&v);
        return Ok(v)
    }
}

/// Consumer for [`join_all`] (super::Event::join_all) event.
#[docfg(feature = "cl1_1")]
pub struct JoinAllConsumer<C> (pub(super) Vec<C>);

#[cfg(feature = "cl1_1")]
impl<'a, T, C: Consumer<'a, T>> Consumer<'a, Vec<T>> for JoinAllConsumer<C> {
    #[inline]
    fn consume (self) -> Result<Vec<T>> {
        self.0.into_iter().map(Consumer::consume).try_collect()
    }
}