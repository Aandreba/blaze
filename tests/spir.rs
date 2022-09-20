use blaze_rs::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn test () {
    let (program, kernels) = RawProgram::from_binary(include_bytes!("llvmir.spv"), None).unwrap();
    println!("{program:?}, {kernels:?}");
}