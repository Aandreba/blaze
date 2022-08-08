use blase::vec::Vector;
use blaze_proc::global_context;
use blaze_rs::prelude::{Result, SimpleContext};

#[global_context]
static CTX : SimpleContext = SimpleContext::default();

#[test]
fn add () -> Result<()> {
    let alpha = Vector::new(&[1, 2, 3, 4, 5], false)?;
    let beta = (2 * alpha) / 3;

    println!("{beta:?}");
    Ok(())
}