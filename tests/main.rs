use rscl::{context::SimpleContext, prelude::Result, image::{Image2D, channel::Luma}, buffer::flags::MemAccess};
use rscl_proc::global_context;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn program () -> Result<()> {
    let img = Image2D::<Luma<f32>>::from_file("tests/test2.jpg", MemAccess::default(), false)?;
    
    Ok(())
}