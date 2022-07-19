#![feature(box_into_inner, new_uninit, extend_one, const_nonnull_new, int_roundings, const_option_ext, maybe_uninit_uninit_array, const_option, nonzero_ops, const_slice_from_raw_parts, ptr_metadata, is_some_with, fn_traits, vec_into_raw_parts, allocator_api)]
#![cfg_attr(any(feature = "svm", feature = "map"), feature(strict_provenance, layout_for_ptr))]
#![cfg_attr(feature = "atomics", feature(cfg_target_has_atomic, core_intrinsics))]
#![cfg_attr(docsrs, feature(doc_cfg, proc_macro_hygiene))]
#![cfg_attr(debug_assertions, feature(backtrace, backtrace_frames))]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

macro_rules! lazy_static {
    ($($vis:vis static ref $name:ident : $ty:ty = $expr:expr;)+) => {
        $(
            $vis static $name : ::once_cell::sync::Lazy<$ty> = ::once_cell::sync::Lazy::new(|| $expr);
        )+
    };
}

macro_rules! tri {
    ($e:expr) => {{
        let err = $e;
        if err != 0 {
            return Err($crate::core::Error::from(err));
        }
    }};

    ($($e:expr);+) => {{
        let mut err;
        $(
            err = $e;
            if err != 0 {
                return Err($crate::core::Error::from(err));
            }
        )+
    }};
}

macro_rules! tri_panic {
    ($e:expr) => {{
        let err = $e;
        if err != 0 {
            panic!("{:?}", $crate::core::Error::from(err))
        }
    }};
}

pub mod prelude {
    pub use crate::core::*;
    pub use crate::macros::*;
    pub use crate::context::{Context, Global, RawContext};
    pub use crate::event::{RawEvent, Event, EventExt};
}

#[doc(hidden)]
pub extern crate once_cell;

extern crate rscl_proc;
pub mod macros {
    pub use rscl_proc::{global_context, rscl};
}

/// Core OpenCL types
pub mod core;
/// RSCL context's
pub mod context;
/// Generic memory object
pub mod memobj;
pub mod buffer;
/// RSCL's event system
pub mod event;
mod utils;

#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
#[cfg(feature = "image")]
pub mod image;

#[cfg_attr(docsrs, doc(cfg(feature = "svm")))]
#[cfg(feature = "svm")]
pub mod svm;