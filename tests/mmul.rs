#![feature(new_uninit)]

use std::{mem::MaybeUninit, f32::consts::{PI, E}};
use rscl::{prelude::{Device}, buffer::BufferRect2D};
use rscl::{context::SimpleContext, prelude::Result, buffer::{flags::MemAccess}, event::WaitList};
use rscl_proc::{global_context, rscl};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

static CODE : &str = "
#define IDX (x, y, width) y * width + x  

kernel void mmul (const uint k, __constant float* lhs, __constant float* rhs, __global float* out) {
    const uint width = get_global_size(1);
    const uint x = get_global_id(0);
    const uint y = get_global_id(1);

    float sum = 0.0f;
    for (uint i = 0; i < k; ++i) {
        sum = fma(lhs[y * k + i], rhs[i * width + x], sum);
    }

    out[y * width + x] = sum;
}
";

#[rscl(MatrixOps)]
#[link(CODE)]
extern {
    fn mmul (k: u32, lhs: *const f32, rhs: *const f32, out: *mut MaybeUninit<f32>);
}

#[test]
fn matrix_mul () -> Result<()> {
    println!("{:?}", Device::all());
    let ops = MatrixOps::new(None)?;

    let lhs = BufferRect2D::<f32>::new(&[PI,2.,4.,5.,7.,8.], 2, MemAccess::READ_ONLY, false)?; // 3 x 2
    let rhs = BufferRect2D::<f32>::new(&[1.,E,3.,4.,5.,6.], 3, MemAccess::READ_ONLY, false)?; // 2 x 3
    let mut result = BufferRect2D::<f32>::uninit(3, 3, MemAccess::WRITE_ONLY, false)?; // 3 x 3

    let evt = unsafe { ops.mmul(2, &lhs, &rhs, &mut result, [3, 3], None, WaitList::EMPTY)? };
    //evt.wait()?;

    let result = unsafe { result.assume_init() };
    println!("{:?}", result);
    
    Ok(())
}