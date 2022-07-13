use std::ops::Deref;

/// Represents types that directly or indirectly dereference `T`
pub trait AsDeref<T> {
    fn from_deref (&self) -> &T;
}

impl<T> AsDeref<T> for T {
    #[inline(always)]
    fn from_deref (&self) -> &T {
        self
    }
}

impl<T, A: Deref<Target = impl 'static + AsDeref<T>>> AsDeref<T> for A {
    fn from_deref (&self) -> &T {
        todo!()
    }
}