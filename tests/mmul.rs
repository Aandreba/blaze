#![feature(new_uninit)]

use std::{mem::MaybeUninit, f32::consts::{PI, E}};
use blaze::{prelude::{RawDevice}, buffer::rect::{BufferRect2D, SvmRect2D}, svm::Svm};
use blaze::{prelude::Result, buffer::{flags::MemAccess}, event::WaitList};
use blaze::prelude::{global_context, blaze};

//#[global_context]
//static CONTEXT : SimpleContext = SimpleContext::default();

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

#[blaze(MatrixOps)]
#[link(CODE)]
extern "C" {
    fn mmul (k: u32, lhs: *const f32, rhs: *const f32, out: *mut MaybeUninit<f32>);
}

#[test]
fn matrix_mul () -> Result<()> {
    println!("{:?}", RawDevice::all());
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

#[test]
fn svm_mul () -> Result<()> {
    println!("{:?}", RawDevice::all());
    let ops = MatrixOps::new(None)?;

    let lhs = BufferRect2D::<f32>::new(&[PI,2.,4.,5.,7.,8.], 2, MemAccess::READ_ONLY, false)?; // 3 x 2
    let rhs = BufferRect2D::<f32>::new(&[1.,E,3.,4.,5.,6.], 3, MemAccess::READ_ONLY, false)?; // 2 x 3
    let mut result = SvmRect2D::<f32, _>::new_uninit_in(3, 3, Svm::new(true)).unwrap();

    let evt = unsafe { ops.mmul(2, &lhs, &rhs, &mut result, [3, 3], None, WaitList::EMPTY)? };
    //evt.wait()?;

    let result = unsafe { result.assume_init() };
    println!("{:?}", result);
    
    Ok(())
}

mod test {
    use std::{sync::Arc};
    use blaze::{prelude::*, context::SimpleContext};

    #[global_context]
    static CONTEXT : SimpleContext = SimpleContext::default();

    #[test]
    fn main () -> Result<()> {
        let buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::READ_ONLY, false).map(Arc::new)?;
        let buffer2 = Buffer::new(&[5, 4, 3, 2, 1], MemAccess::WRITE_ONLY, false).map(Arc::new)?;

        let read = buffer.read_all_owned(EMPTY)?;
        let read2 = buffer2.read_all_owned(&read)?;
        let join = ReadBuffer::join_ordered([read2, read])?.wait()?;

        assert_eq!(join[0].as_slice(), &[5, 4, 3, 2, 1]);
        assert_eq!(join[1].as_slice(), &[1, 2, 3, 4, 5]);
        Ok(())
    }

    fn without_global () -> Result<()> {
        let ctx = SimpleContext::default()?;
        
        let buffer = Buffer::new_in(ctx.clone(), &[1, 2, 3, 4, 5], MemAccess::READ_ONLY, false).map(Arc::new)?;
        let buffer2 = Buffer::new_in(ctx.clone(), &[5, 4, 3, 2, 1], MemAccess::WRITE_ONLY, false).map(Arc::new)?;

        let read = buffer.read_all_owned(EMPTY)?;
        let read2 = buffer2.read_all_owned(&read)?;
        let join = ReadBuffer::join_ordered_in(&ctx, [read2, read])?.wait()?;

        assert_eq!(join[0].as_slice(), &[5, 4, 3, 2, 1]);
        assert_eq!(join[1].as_slice(), &[1, 2, 3, 4, 5]);
        Ok(())
    }
}