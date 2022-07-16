use std::{ops::Deref};
use crate::{buffer::Buffer, prelude::Context};

/// Represents types that directly or indirectly dereference `T`
pub trait AsDeref<T> {
    fn from_deref (&self) -> &T;
}
 
impl<T, A: Deref<Target = T>> AsDeref<T> for A {
    fn from_deref (&self) -> &T {
        todo!()
    }
}

impl<T, A: Deref<Target = impl 'static + AsDeref<T>>> AsDeref<T> for A {
    fn from_deref (&self) -> &T {
        todo!()
    }
}

#[test]
fn test () {
    let test : &dyn AsDeref<f32> = &std::sync::Arc::new(1.);
}