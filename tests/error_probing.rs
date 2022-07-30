use blaze::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let flag = FlagEvent::new()?;
    let (program, kernels) = RawProgram::from_source("kernel void test () {}", None)?;
    let kernel = &kernels[0];
    let program2 = kernel.program()?;
    
    println!("{}", program.reference_count()?);
    Ok(())
}