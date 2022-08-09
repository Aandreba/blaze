use std::f64::consts::PI;

use blase::{vec::Vector, random::Random};
use blaze_proc::global_context;
use blaze_rs::prelude::{Result, SimpleContext, EMPTY, Event};

#[global_context]
static CTX : SimpleContext = SimpleContext::default();

#[test]
fn add () -> Result<()> {
    let alpha = Vector::new(&[1, 2, 3, 4, 5], false)?;
    let beta = (2 * alpha) / 3;

    println!("{beta:?}");
    Ok(())
}

#[test]
fn sum () -> Result<()> {
    let mut rng = Random::new(None)?;
    let buffer = Vector::from_buffer(rng.next_f32(1003, 0f32..1f32, true, false)?);
    let slice = buffer.map(.., EMPTY)?.wait()?;

    let cpu_sum = slice.into_iter().sum::<f32>();
    let gpu_sum = buffer.sum(EMPTY)?.wait()?;

    println!("{cpu_sum} v. {gpu_sum}");
    Ok(())
}