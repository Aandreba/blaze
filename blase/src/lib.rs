#![cfg_attr(docsrs, feature(doc_cfg))]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

flat_mod!(r#trait);
pub mod vec;

pub(crate) fn include_prog<T: Real> (src: &str) -> String {
    let mut exts = String::new();
    for ext in T::EXTENSIONS.into_iter() {
        exts.push_str(&format!("#pragma OPENCL EXTENSION {ext}: enable\n"));
    }

    format!(
        "{exts}
        #define PRECISION {}
        #define ISFLOAT {}
        typedef {} real;
        {src}",
        T::PRECISION,
        T::FLOAT,
        T::CL_NAME
    )
}

#[test]
fn test_prog () {
    let prog = include_prog::<f32>(include_str!("opencl/vec.cl"));
    println!("{prog}");
}