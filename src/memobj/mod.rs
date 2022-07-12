flat_mod!(raw, flags);

#[cfg(feature = "map")]
mod map;
#[cfg(feature = "map")]
#[cfg_attr(docsrs, doc(cfg(feature = "map")))]
pub use map::*;