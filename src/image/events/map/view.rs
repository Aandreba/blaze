use std::ops::Deref;
use crate::image::channel::RawPixel;

pub struct MapView<P: RawPixel, D> {
    inner: D,
    a: P
}

impl<P: RawPixel, D: Deref<Target = [P::Subpixel]>> MapView<P, D> {
    pub fn new (inner: D, row_pitch: usize) -> Self {
        todo!()
    }
}