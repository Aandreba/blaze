use std::{thread::sleep, time::Duration};

use blase::{vec::EucVec, random::Random};
use blaze_rs::{prelude::*, event::FlagEvent, wait_list_from_ref};

#[global_context]
static CTX : SimpleContext = SimpleContext::default();

#[test]
fn add () -> Result<()> {
    let mut random = Random::new(None)?;
    let alpha = EucVec::from_buffer(random.next_f32_blocking(10, 0.0..1.0, true, false)?);
    let beta = EucVec::new(&[1., 2., 3., 4., 5.], false)?;

    println!("{alpha:?} - {beta:?}");

    Ok(())
}

#[test]
fn sum () -> Result<()> {
    let mut random = Random::new(None)?;

    let alpha = EucVec::from_buffer(random.next_f32_blocking(5, -5.0..=5.0, true, false)?);
    let gamma = alpha.magn_blocking(None)?;

    println!("sqrt({alpha:?} * {alpha:?}) = {gamma}");
    Ok(())
}

/*
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

    let alpha = EucVec::from_buffer(rng.next_u8(LEN, 0..5, true, false)?);
    let beta = EucVec::from_buffer(rng.next_u8(LEN, 0..5, true, false)?);
    
    let (eq, len) = alpha.lane_eq(&beta, EMPTY)?.wait()?;
    let eq = eq.into_iter()
        .take(len)
        .enumerate()
        .filter_map(|(i, x)| x.then(|| i));
    
    for idx in eq {
        println!("{idx}");
    }
    
    Ok(())
}

#[test]
fn ord () -> Result<()>{
    let alpha = EucVec::new(&[1.0, 2.0, 3.0, 4.0, f32::NAN], false)?;
    let beta = EucVec::new(&[2.6, -5e-8, f32::NAN, 8.29, f32::INFINITY, -1.0, 0.0, -f32::INFINITY, -0.0], false)?;
    let beta = beta.sort(true, EMPTY)?.wait()?;
    println!("{beta:?}");

    let ord = alpha.lane_ord(&beta, EMPTY)?.wait()?;
    let partial = alpha.lane_partial_ord(&beta, EMPTY)?.wait()?;
    println!("{ord:?}, {partial:?}");

    Ok(())
}*/