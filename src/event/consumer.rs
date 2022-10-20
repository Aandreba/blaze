use std::{marker::PhantomData, panic::{UnwindSafe, catch_unwind}, any::Any};
use blaze_proc::docfg;
use crate::prelude::Result;

/// A trait that represents the consumer of an [`Event`](super::Event)
pub trait Consumer {
    type Output;

    /// Consumes the [`Consumer`].
    /// 
    /// # Safety
    /// This method should be safe to execute whenever it's underlying [`RawEvent`](super::RawEvent) has completed.
    /// Execution of this method before the event's completion is undefined behaviour.
    unsafe fn consume (self) -> Result<Self::Output>;
}

impl<T, F: FnOnce() -> Result<T>> Consumer for F {
    type Output = T;

    #[inline(always)]
    unsafe fn consume (self) -> Result<T> {
        (self)()
    }
}

impl<T: ?Sized> Consumer for PhantomData<T> {
    type Output = ();

    #[inline(always)]
    unsafe fn consume (self) -> Result<Self::Output> {
        Ok(())
    }
}

impl<T> Consumer for Result<T> {
    type Output = T;

    #[inline(always)]
    unsafe fn consume (self) -> Result<T> {
        self
    }
}

/// Consumer for [`specific`](super::Event::specific) event
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Specific<'a, C: 'a> (C, PhantomData<&'a mut &'a ()>);

impl<C> Specific<'_, C> {
    #[inline(always)]
    pub fn new (c: C) -> Self { Self(c, PhantomData) }
}

impl<'a, C: 'a + Consumer> Consumer for Specific<'a, C> {
    type Output = C::Output;

    #[inline(always)]
    unsafe fn consume (self) -> Result<Self::Output> {
        self.0.consume()
    }
}

/// A **no**-**op**eration consumer
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Noop;

impl Consumer for Noop {
    type Output = ();

    #[inline(always)]
    unsafe fn consume (self) -> Result<Self::Output> {
        Ok(())
    }
}

/// Consumer for [`map`](super::Event::map) event.
#[derive(Debug, Clone)]
pub struct Map<T, C, F> (pub(crate) C, pub(crate) F, PhantomData<T>);

impl<T, U, C: Consumer<Output = T>, F: FnOnce(T) -> U> Map<T, C, F> {
    #[inline(always)]
    pub const fn new (consumer: C, f: F) -> Self { Self(consumer, f, PhantomData) } 
}

impl<T, U, C: Consumer<Output = T>, F: FnOnce(T) -> U> Consumer for Map<T, C, F> {
    type Output = U;

    #[inline(always)]
    unsafe fn consume (self) -> Result<U> {
        let v = self.0.consume()?;
        return Ok((self.1)(v))
    }
}

/// Consumer for [`try_map`](super::Event::try_map) event.
#[derive(Debug, Clone)]
pub struct TryMap<T, C, F> (pub(crate) C, pub(crate) F, PhantomData<T>);

impl<T, U, C: Consumer<Output = T>, F: FnOnce(T) -> Result<U>> TryMap<T, C, F> {
    #[inline(always)]
    pub const fn new (consumer: C, f: F) -> Self { Self(consumer, f, PhantomData) } 
}

impl<T, U, C: Consumer<Output = T>, F: FnOnce(T) -> Result<U>> Consumer for TryMap<T, C, F> {
    type Output = U;

    #[inline(always)]
    unsafe fn consume (self) -> Result<U> {
        let v = self.0.consume()?;
        return (self.1)(v)
    }
}

/// Consumer for [`catch_unwind`](super::Event::catch_unwind) event.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct CatchUnwind<C: UnwindSafe> (pub(super) C);

impl<C: Consumer + UnwindSafe> Consumer for CatchUnwind<C> {
    type Output = ::core::result::Result<C::Output, Box<dyn Any + Send>>;

    #[inline(always)]
    unsafe fn consume (self) -> Result<Self::Output> {
        return match catch_unwind(|| self.0.consume()) {
            Ok(Ok(x)) => Ok(Ok(x)),
            Ok(Err(e)) => Err(e),
            Err(e) => Ok(Err(e))
        }
    }
}

/// Consumer for [`flatten_result`](super::Event::flatten_result) event.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct FlattenResult<C> (pub(super) C);

impl<T, C: Consumer<Output = Result<T>>> Consumer for FlattenResult<C> {
    type Output = T;

    #[inline(always)]
    unsafe fn consume (self) -> Result<T> {
        self.0.consume().flatten()
    }
}

/// Consumer for [`inspect`](super::Event::inspect) event.
#[derive(Debug, Clone)]
pub struct Inspect<C, F> (pub(super) C, pub(super) F);

impl<C: Consumer, F: FnOnce(&C::Output)> Consumer for Inspect<C, F> {
    type Output = C::Output;

    #[inline(always)]
    unsafe fn consume (self) -> Result<C::Output> {
        let v = self.0.consume()?;
        (self.1)(&v);
        return Ok(v)
    }
}

/// Consumer for [`join_all`](super::Event::join_all) event.
#[docfg(feature = "cl1_1")]
#[derive(Debug, Clone)]
pub struct JoinAll<C> (pub(super) Vec<C>);

#[cfg(feature = "cl1_1")]
impl<C: Consumer> Consumer for JoinAll<C> {
    type Output = Vec<C::Output>;

    #[inline]
    unsafe fn consume (self) -> Result<Vec<C::Output>> {
        self.0.into_iter().map(|x| x.consume()).try_collect()
    }
}