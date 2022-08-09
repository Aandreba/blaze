#![allow(non_upper_case_globals)]

use blaze_proc::docfg;
use crate::vec::VectorProgram;

#[cfg(feature = "half")]
static half : once_cell::sync::Lazy<VectorProgram<::half::f16>> = once_cell::sync::Lazy::new(|| VectorProgram::new(None).unwrap());
static float : once_cell::sync::Lazy<VectorProgram<f32>> = once_cell::sync::Lazy::new(|| VectorProgram::new(None).unwrap());
#[cfg(feature = "double")]
static double : once_cell::sync::Lazy<VectorProgram<f64>> = once_cell::sync::Lazy::new(|| VectorProgram::new(None).unwrap());

pub trait Real: 'static + Copy + Send + Sync + Unpin {
    const CL_NAME : &'static str;
    const EXTENSIONS : &'static [&'static str];
    const PRECISION : u32;
    const FLOAT : bool;
    const SIGNED : bool;

    fn vec_program () -> &'static VectorProgram<Self>;
}

#[docfg(feature = "half")]
impl Real for ::half::f16 {
    const CL_NAME : &'static str = "half";
    const EXTENSIONS : &'static [&'static str] = &["cl_khr_fp16"];
    const PRECISION : u32 = 16;
    const FLOAT : bool = true;
    const SIGNED : bool = true;
    
    #[inline(always)]
    fn vec_program () -> &'static VectorProgram<Self> { &half }
}

impl Real for f32 {
    const CL_NAME : &'static str = "float";
    const EXTENSIONS : &'static [&'static str] = &[];
    const PRECISION : u32 = 32;
    const FLOAT : bool = true;
    const SIGNED : bool = true;

    #[inline(always)]
    fn vec_program () -> &'static VectorProgram<Self> { &float }
}

#[docfg(feature = "double")]
impl Real for f64 {
    const CL_NAME : &'static str = "double";
    const EXTENSIONS : &'static [&'static str] = &["cl_khr_fp64"];
    const PRECISION : u32 = 64;
    const FLOAT : bool = true;
    const SIGNED : bool = true;

    #[inline(always)]
    fn vec_program () -> &'static VectorProgram<Self> { &double }
}

macro_rules! impl_int {
    ($($i:ty as $name:ident),+) => {
        $(
            static $name : once_cell::sync::Lazy<VectorProgram<$i>> = once_cell::sync::Lazy::new(|| VectorProgram::new(None).unwrap());

            impl Real for $i {
                const CL_NAME : &'static str = stringify!($name);
                const EXTENSIONS : &'static [&'static str] = &[];
                const PRECISION : u32 = <$i>::BITS;
                const FLOAT : bool = false;
                const SIGNED : bool = stringify!($i).as_bytes()[0] == b'i';
                
                #[inline(always)]
                fn vec_program () -> &'static VectorProgram<Self> { &$name }
            }
        )+
    };
}

impl_int! {
    u8 as uchar,
    u16 as ushort,
    u32 as uint,
    u64 as ulong,
    
    i8 as char,
    i16 as short,
    i32 as int,
    i64 as long
}