#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();
use std::mem::MaybeUninit;
use blaze_rs::{prelude::*};

#[allow(dead_code)]
static CODE : &str = "
    #define IDX (x, y, width) y * width + x  

    kernel void mul (const uint k, __constant float* lhs, __constant float* rhs, __global float* out) {
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
#[link = CODE]
extern "C" {
    #[link_name = "mul"]
    fn matrix_mul (k: u32, lhs: *const f32, rhs: *const f32, out: *mut MaybeUninit<f32>);
}

#[test]
fn buffer_mul () -> Result<()> {
    let ops = MatrixOps::new(None)?;

    let lhs = BufferRect2D::<f32>::new(&[1.,2.,4.,5.,7.,8.], 2, MemAccess::READ_ONLY, false)?; // 3 x 2
    let rhs = BufferRect2D::<f32>::new(&[1.,2.,3.,4.,5.,6.], 3, MemAccess::READ_ONLY, false)?; // 2 x 3
    let mut result = BufferRect2D::<f32>::new_uninit(3, 3, MemAccess::WRITE_ONLY, false)?; // 3 x 3

    scope(|s| unsafe {
        ops.matrix_mul(s, 2, &lhs, &rhs, &mut result, [3, 3], None, &[])?.join()?;
        Ok(())
    })?;

    let result = unsafe { result.assume_init() };
    println!("{:?}", result);
    
    Ok(())
}

#[cfg(feature = "svm")]
#[test]
fn svm_mul () -> Result<()> {
    /*use blaze_rs::{buffer::rect::SvmRect2D, svm::Svm};
    let ops = MatrixOps::new(None)?;

    let lhs = BufferRect2D::<f32>::new(&[1.,2.,4.,5.,7.,8.], 2, MemAccess::READ_ONLY, false)?; // 3 x 2
    let rhs = BufferRect2D::<f32>::new(&[1.,2.,3.,4.,5.,6.], 3, MemAccess::READ_ONLY, false)?; // 2 x 3
    let mut result = SvmRect2D::<f32>::new_uninit_in(3, 3, Svm::default()).unwrap(); // 3 x 3

    let evt = unsafe { ops.matrix_mul(2, &lhs, &rhs, &mut result, [3, 3], None, WaitList::EMPTY)? };
    evt.wait()?;

    let result = unsafe { result.assume_init() };
    println!("{:?}", result);
    
    Ok(())*/
    Ok(())
}