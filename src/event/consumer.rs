use crate::prelude::Result;

/// A trait that represents the consumer of an [`Event`]
pub trait Consumer<'a, T>: 'a {
    fn consume (self) -> Result<T>;
}

impl<'a, T, F: 'a + FnOnce() -> Result<T>> Consumer<'a, T> for F {
    #[inline(always)]
    fn consume (self) -> Result<T> {
        self()
    }
}

impl<'a, T, F: 'a + FnOnce() -> Result<T>> Consumer<'a, T> for Box<F> {
    #[inline(always)]
    fn consume (self) -> Result<T> {
        self()
    }
}

/// Noop trait consumer
pub struct Noop;

impl Consumer<'_, ()> for Noop {
    #[inline(always)]
    fn consume (self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test () {
    let f = || Ok(());
    let f : Box<dyn Consumer<'static, ()>> = Box::new(f);
 

    todo!()
}