use blase::vec::Vector;
use blaze_proc::global_context;
use blaze_rs::prelude::{Result, SimpleContext};

#[global_context]
static CTX : SimpleContext = SimpleContext::default();

#[test]
fn add () -> Result<()> {
    let alpha = Vector::new(&[1, 2, 3, 4, 5], false)?;
    let beta = Vector::new(&[6, 7, 8, 9, 10], false)?;
    let test = (alpha + beta) * 2;

    println!("{test:?}");
    Ok(())
}