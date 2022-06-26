#![feature(new_uninit)]

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
}

macro_rules! tri_panic {
    ($e:expr) => {{
        let err = $e;
        if err != 0 {
            panic!("{:?}", $crate::core::Error::from(err))
        }
    }};
}

#[doc(hidden)]
pub extern crate once_cell;

extern crate rscl_proc;
pub mod macros {
    pub use rscl_proc::*;
}

/// Core OpenCL types
pub mod core;
/// RSCL context's
pub mod context;
pub mod kernel;
pub mod buffer;