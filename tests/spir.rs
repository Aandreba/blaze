use blaze_rs::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[cfg(feature = "cl2_1")]
#[test]
fn test () {
    let (program, kernels) = RawProgram::from_il(include_bytes!("llvmir.spv"), None).unwrap();
    println!("{program:?}, {kernels:?}");
}