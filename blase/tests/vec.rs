use std::f64::consts::PI;

use blase::{vec::EucVec, random::Random};
use blaze_proc::global_context;
use blaze_rs::{prelude::{Result, SimpleContext, EMPTY, Event, EventExt}, buffer::events::MapBuffer};

#[global_context]
static CTX : SimpleContext = SimpleContext::default();

#[test]
fn add () -> Result<()> {
    let alpha = EucVec::new(&[1, 2, 3, 4, 5], false)?;
    let beta = (2 * alpha) / 3;

    println!("{beta:?}");
    Ok(())
}

#[test]
fn sum () -> Result<()> {
    let mut rng = Random::new(None)?;
    let buffer = EucVec::from_buffer(rng.next_f32(1003, 0f32..1f32, true, false)?);
    let slice = buffer.map(.., EMPTY)?.wait()?;

    let cpu_sum = slice.into_iter().sum::<f32>();
    let gpu_sum = buffer.sum(EMPTY)?.wait()?;

    println!("{cpu_sum} v. {gpu_sum}");
    Ok(())
}

#[test]
fn dot () -> Result<()> {
    const LEN : usize = 100;
    let mut rng = Random::new(None)?;

    let alpha = EucVec::from_buffer(rng.next_f32(LEN, 0f32..1f32, true, false)?);
    let beta = EucVec::from_buffer(rng.next_f32(LEN, 0f32..1f32, true, false)?);
    
    let alpha_slice = alpha.map(.., EMPTY)?;
    let beta_slice = beta.map(.., EMPTY)?;
    let join = MapBuffer::join_blocking([alpha_slice, beta_slice])?;

    let gpu_dot = &alpha * &beta;
    let cpu_dot = join[0].into_iter()
        .zip(join[1].into_iter())
        .map(|(x, y)| x * y)
        .sum::<f32>(); 

    println!("{gpu_dot} v. {cpu_dot}");
    Ok(())
}