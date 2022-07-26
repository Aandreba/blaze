use std::{io::Write, ops::{DerefMut, Deref}};
use crate::{image::channel::RawPixel, buffer::rect::Rect2D};

pub struct PngEncoder<P, W> {
    pixels: P,
    target: W,
    filter: Filtering
}

impl<T: RawPixel, P: Deref<Target = T>, W: DerefMut> PngEncoder<P, W> where W::Target: Write {
    #[inline(always)]
    pub const fn new (pixels: P, target: W, filter: Filtering) -> Self {
        Self { pixels, target, filter }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum Filtering {
    /// The raw byte value passes through unaltered
    None = 0,
    /// Predicted to the left
    Sub = 1,
    /// Predicted above
    Up = 2,
    /// Mean of left and upper bytes, rounded down 
    #[default]
    Average = 3,
    /// Left, upper, or upper-left, whichever is closest to p = left + upper − upper-left
    Paeth = 4
}

impl Filtering {
    fn filter<T: RawPixel> (&self, rect: &Rect2D<T>, x: usize, y: usize) -> T {
        match self {
            Self::None => rect.get_copy(y, x).unwrap(),
            _ => todo!()
        }
    }
}