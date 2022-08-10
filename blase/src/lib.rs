#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(nonzero_min_max, exclusive_range_pattern, int_roundings, new_uninit)]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

macro_rules! lazy_static {
    ($($v:vis static ref $name:ident: $t:ty = $e:expr;)+) => {
        $(
            $v static $name : ::once_cell::sync::Lazy<$t> = ::once_cell::sync::Lazy::new(|| $e);
        )+
    };
}

flat_mod!(r#trait, ctx);
pub mod vec;
pub mod random;
pub mod utils;

pub(crate) fn include_prog<T: Real> (src: &str) -> String {
    let mut exts = String::new();
    for ext in T::EXTENSIONS.into_iter() {
        exts.push_str(&format!("#pragma OPENCL EXTENSION {ext}: enable\n"));
    }

    format!(
        "{exts}
        #define PRECISION {}
        #define ISFLOAT {}
        #define ISSIGNED {}
        #define FMA(a,b,c) {}
        {5}
        typedef {} real;
        {src}",
        T::PRECISION,
        T::SIGNED,
        T::FLOAT,
        T::FMA,
        T::CL_NAME,
        define_usize()
    )
}

#[inline(always)]
pub(crate) fn define_usize () -> String {
    cfg_if::cfg_if! {
        if #[cfg(target_pointer_width = "8")] {
            const USIZE : &'static str = "uchar";
        } else if #[cfg(target_pointer_width = "16")] {
            const USIZE : &'static str = "ushort";
        } else if #[cfg(target_pointer_width = "32")] {
            const USIZE : &'static str = "uint";
        } else if #[cfg(target_pointer_width = "64")] {
            const USIZE : &'static str = "ulong";
        } else {
            compile_error!("Unsupported pointer width");
        }
    }

    format!("typedef {USIZE} usize;")
}

#[test]
fn test_prog () {
    let prog = include_prog::<f32>(include_str!("opencl/vec.cl"));
    println!("{prog}");
}