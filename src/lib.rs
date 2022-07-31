#![feature(box_into_inner, new_uninit, iterator_try_collect, extend_one, const_nonnull_new, int_roundings, const_maybe_uninit_zeroed, const_ptr_as_ref, const_maybe_uninit_array_assume_init, maybe_uninit_array_assume_init, const_option_ext, maybe_uninit_uninit_array, const_option, nonzero_ops, associated_type_bounds, ptr_metadata, is_some_with, fn_traits, vec_into_raw_parts, const_trait_impl, allocator_api)]
#![cfg_attr(any(feature = "svm", feature = "map"), feature(strict_provenance, layout_for_ptr))]
#![cfg_attr(docsrs, feature(doc_cfg, proc_macro_hygiene))]
#![cfg_attr(debug_assertions, feature(backtrace, backtrace_frames))]
#![doc = include_str!("../docs/src/intro.md")]

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

    ($($e:expr);+) => {{
        let mut err;
        $(
            err = $e;
            if err != 0 {
                panic!("{:?}", $crate::core::Error::from(err))
            }
        )+
    }};
}

pub mod prelude {
    pub const EMPTY : WaitList = WaitList::EMPTY;

    pub use crate::core::*;
    pub use crate::macros::*;
    pub use crate::buffer::{RawBuffer, Buffer, flags::*, events::{ReadBuffer, WriteBuffer, CopyBuffer}};
    pub use crate::context::{Context, Global, RawContext, SimpleContext};
    pub use crate::event::{RawEvent, Event, EventExt, WaitList};
    pub use crate::memobj::RawMemObject;
    #[blaze_proc::docfg(feature = "cl1_1")]
    pub use crate::event::FlagEvent;
    pub use crate::buffer::rect::{BufferRect2D, Rect2D};
    #[blaze_proc::docfg(feature = "cl1_1")]
    pub use crate::buffer::rect::{ReadBufferRect2D, WriteBufferRect2D};
}

#[doc(hidden)]
pub extern crate once_cell;

extern crate blaze_proc;
pub mod macros {
    pub use blaze_proc::{global_context, blaze};
}

#[doc = include_str!("../docs/src/raw.md")]
pub mod core;
#[doc = include_str!("../docs/src/context/README.md")]
pub mod context;
/// Generic memory object
pub mod memobj;
/// Blaze buffers
pub mod buffer;
#[doc = include_str!("../docs/src/events/README.md")]
pub mod event;

#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
#[cfg(feature = "image")]
pub mod image;

#[doc = include_str!("../docs/src/svm/README.md")]
#[cfg_attr(docsrs, doc(cfg(feature = "svm")))]
#[cfg(feature = "svm")]
pub mod svm;