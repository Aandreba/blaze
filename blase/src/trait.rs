use blaze_proc::docfg;

pub trait Real: 'static + Copy + Send + Sync + Unpin {
    const CL_NAME : &'static str;
    const EXTENSIONS : &'static [&'static str];
    const PRECISION : u32;
    const FLOAT : bool;

    const ADD_MACRO : &'static str = "x + y";
    const SUB_MACRO : &'static str = "x - y";
}

#[docfg(feature = "half")]
impl Real for ::half::f16 {
    const CL_NAME : &'static str = "half";
    const EXTENSIONS : &'static [&'static str] = &["cl_khr_fp16"];
    const PRECISION : u32 = 16;
    const FLOAT : bool = true;
}

impl Real for f32 {
    const CL_NAME : &'static str = "float";
    const EXTENSIONS : &'static [&'static str] = &[];
    const PRECISION : u32 = 32;
    const FLOAT : bool = true;
}

#[docfg(feature = "double")]
impl Real for f64 {
    const CL_NAME : &'static str = "double";
    const EXTENSIONS : &'static [&'static str] = &["cl_khr_fp64"];
    const PRECISION : u32 = 64;
    const FLOAT : bool = true;
}

macro_rules! impl_int {
    ($($i:ty as $name:literal),+) => {
        $(
            impl Real for $i {
                const CL_NAME : &'static str = $name;
                const EXTENSIONS : &'static [&'static str] = &[];
                const PRECISION : u32 = <$i>::BITS;
                const FLOAT : bool = false;
            }
        )+
    };
}


impl_int! {
    u8 as "uchar",
    u16 as "ushort",
    u32 as "uint",
    u64 as "ulong",

    i8 as "char",
    i16 as "short",
    i32 as "int",
    i64 as "long"
}