use blaze_rs::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn test () {
    let (program, kernels) = RawProgram::from_source("kernel void test () {
        printf(\"Hello\");
    }", None).unwrap();
    println!("{program:?}, {kernels:?}");
}