flat_mod!(raw, complex, range);
#[cfg(feature = "cl1_1")]
pub use rect::BufferRect2D;

#[cfg_attr(docsrs, doc(cfg(feature = "cl1_1")))]
#[cfg(feature = "cl1_1")]
pub mod rect;
pub mod flags;
pub mod events;