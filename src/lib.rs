#![allow(clippy::all)]
#![allow(clippy::needless_return)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
/* */
#![cfg_attr(
    feature = "nightly",
    feature(new_uninit, const_nonnull_new, array_try_map)
)]
#![cfg_attr(feature = "svm", feature(allocator_api, strict_provenance))]
#![cfg_attr(docsrs, feature(doc_cfg, proc_macro_hygiene))]
#![cfg_attr(debug_assertions, feature(backtrace_frames))]
#![doc = include_str!("../docs/src/intro.md")]

use std::ptr::NonNull;

use event::RawEvent;

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

mod blaze_rs {
    pub use crate::*;
}

pub mod prelude {
    pub use crate::buffer::rect::{RectBox2D, RectBuffer2D};
    pub use crate::buffer::{flags::*, Buffer, RawBuffer};
    pub use crate::context::{scope, Context, Global, RawContext, Scope, SimpleContext};
    pub use crate::core::*;
    pub use crate::event::{Event, RawEvent};
    pub use crate::macros::*;
    pub use crate::memobj::RawMemObject;
    pub use crate::WaitList;
}

#[doc(hidden)]
pub extern crate once_cell;

#[cfg(feature = "futures")]
#[doc(hidden)]
pub extern crate futures;

#[cfg(feature = "futures")]
#[doc(hidden)]
pub extern crate utils_atomics;

#[doc(hidden)]
pub extern crate blaze_proc;

/// Re-export of the public-facing macros in `blaze_proc`
pub mod macros {
    #[doc = include_str!("../docs/src/program/README.md")]
    pub use blaze_proc::blaze;
    pub use blaze_proc::global_context;

    /// Similar to [`Event::join_all_blocking`](crate::event::Event::join_all_blocking), but it can also join events with different [`Consumer`](crate::event::Consumer)s
    /// ```rust
    /// use blaze_rs::{prelude::*, macros::*};
    /// use std::ops::Deref;
    ///
    /// #[global_context]
    /// static CONTEXT : SimpleContext = SimpleContext::default();
    ///
    /// # fn main () -> Result<()> {
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
    /// # return Ok(())
    /// # }
    /// ```
    pub use blaze_proc::join_various_blocking;
}

/// Blaze buffers
pub mod buffer;
#[doc = include_str!("../docs/src/context/README.md")]
pub mod context;
#[doc = include_str!("../docs/src/raw.md")]
pub mod core;
#[doc = include_str!("../docs/src/events/README.md")]
pub mod event;
/// Generic memory object
pub mod memobj;

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
pub fn wait_list(v: WaitList) -> core::Result<(u32, *const opencl_sys::cl_event)> {
    return match v {
        Some(v) => match v.len() {
            0 => Ok((0, ::core::ptr::null())),
            len => {
                let len = <u32 as std::convert::TryFrom<usize>>::try_from(len)
                    .map_err(|e| core::Error::new(core::ErrorKind::InvalidEventWaitList, e))?;

                return Ok((len, v.as_ptr().cast()));
            }
        },
        None => Ok((0, ::core::ptr::null())),
    };
}

/// Creates a [`WaitList`] from a reference to a single [`RawEvent`]
#[inline(always)]
pub const fn wait_list_from_ref(evt: &RawEvent) -> WaitList {
    return Some(::core::slice::from_ref(evt));
}

/// A list of events to be awaited.
pub type WaitList<'a> = Option<&'a [prelude::RawEvent]>;

pub(crate) fn try_collect<T, E, C: Default + Extend<T>>(
    mut iter: impl Iterator<Item = Result<T, E>>,
) -> Result<C, E> {
    let mut result = C::default();

    loop {
        match iter.next() {
            Some(Ok(x)) => result.extend(Some(x)),
            Some(Err(e)) => return Err(e),
            None => break,
        }
    }

    return Ok(result);
}

pub(crate) const fn non_null_const<T>(ptr: *mut T) -> Option<NonNull<T>> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "nightly")] {
            return NonNull::new(ptr)
        } else {
            if unsafe { ::core::mem::transmute::<*mut T, usize>(ptr) == 0 } {
                return None;
            }
            return unsafe { Some(NonNull::new_unchecked(ptr)) };
        }
    }
}
