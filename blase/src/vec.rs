use blaze_rs::prelude::*;
use crate::{include_prog, Real};

#[blaze(VectorArith)]
#[link = include_prog::<f32>(include_str!("opencl/vec.cl"))]
extern "C" {
    fn add (n: u32, lhs: *const f32, rhs: *const f32, out: *mut f32);
}

pub struct Vector<T: Real, C: Context = Global> {
    inner: Buffer<T, C>
}

impl<T: Real, C: Context> Vector<T, C> {
    fn test () {
        
    }
}