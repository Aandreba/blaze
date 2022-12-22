#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
#![feature(mem_copy_fn, is_some_and, box_into_inner, nonzero_min_max, new_uninit, iterator_try_collect, result_flattening, alloc_layout_extra, array_try_map, extend_one, const_nonnull_new, int_roundings, const_maybe_uninit_zeroed, unboxed_closures, const_ptr_as_ref, const_maybe_uninit_array_assume_init, maybe_uninit_array_assume_init, const_option_ext, maybe_uninit_uninit_array, const_option, nonzero_ops, associated_type_bounds, ptr_metadata, fn_traits, vec_into_raw_parts, const_trait_impl, drain_filter, allocator_api)]
#![cfg_attr(any(feature = "svm", feature = "map"), feature(strict_provenance, layout_for_ptr))]
#![cfg_attr(docsrs, feature(doc_cfg, proc_macro_hygiene))]
#![cfg_attr(debug_assertions, feature(backtrace_frames))]
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

mod blaze_rs { pub use crate::*; }

pub mod prelude {
    pub use crate::core::*;
    pub use crate::macros::*;
    pub use crate::buffer::{RawBuffer, Buffer, flags::*};
    pub use crate::context::{Context, Global, RawContext, SimpleContext, Scope, scope};
    pub use crate::event::{RawEvent, Event};
    pub use crate::memobj::RawMemObject;
    pub use crate::buffer::rect::{RectBuffer2D, RectBox2D};
}

#[doc(hidden)]
pub extern crate once_cell;

#[cfg(feature = "futures")]
#[doc(hidden)]
pub extern crate futures;

#[cfg(feature = "futures")]
#[doc(hidden)]
pub extern crate utils_atomics;

extern crate blaze_proc;

/// Re-export of the public-facing macros in `blaze_proc`
pub mod macros {
    #[doc = include_str!("../docs/src/program/README.md")]
    pub use blaze_proc::blaze;
    pub use blaze_proc::{global_context};

    /// Similar to [`Event::join_all_blocking`](crate::event::Event::join_all_blocking), but it can also join events with different [`Consumer`](crate::event::Consumer)s
    /// ```rust
    /// use std::ops::Deref;  
    ///   
    /// let buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    /// 
    /// let (left, right) = scope(|s| {
    ///     let left = buffer.read(s, 2.., None)?;
    ///     let right = buffer.map(s, 2.., None)?;
    ///     return join_various_blocking!(left, right)
    /// })?;
    /// 
    /// assert_eq!(left.as_slice(), right.deref());
    /// ```
    pub use blaze_proc::join_various_blocking;
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

/// Turns a [`WaitList`] into raw components to be passed to an OpenCL method.
/// # Error
/// This method returns [`ErrorKind::InvalidEventWaitList`](core::ErrorKind::InvalidEventWaitList) if the list's size cannot fit inside a `u32`.
#[inline]
pub fn wait_list (v: WaitList) -> core::Result<(u32, *const opencl_sys::cl_event)> {
    return match v {
        Some(v) => match v.len() {
            0 => Ok((0, ::core::ptr::null())),
            len => {
                let len = <u32 as std::convert::TryFrom<usize>>::try_from(len)
                    .map_err(|e| core::Error::new(core::ErrorKind::InvalidEventWaitList, e))?;

                return Ok((len, v.as_ptr().cast()))
            },
        },
        None => Ok((0, ::core::ptr::null()))
    }
}

/// A list of events to be awaited.
pub type WaitList<'a> = Option<&'a [prelude::RawEvent]>;