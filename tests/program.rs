use blaze_rs::{
    buffer,
    prelude::{blaze, global_context, Buffer, MemAccess, Result, SimpleContext},
};
use std::mem::MaybeUninit;

#[global_context]
static CONTEXT: SimpleContext = SimpleContext::default();

#[blaze(pub FloatTanh)]
#[link = KERNEL]
extern "C" {
    fn forward(n: u64, x_buffer: *mut f32);
    fn backward(n: u64, x_buffer: *const f32, y_buffer: *mut MaybeUninit<f32>);
}

/* MANUALLY */
const KERNEL: &str = r#"
    __kernel void forward (ulong n, __global float* x_buffer) {
        for (ulong i = get_global_id(0); i < n; i += get_global_size(0)) {
            x_buffer[i] = tanh(x_buffer[i]);
        }
    }

    __kernel void backward (ulong n, const __global float* x_buffer, __global float* y_buffer) {
        for (ulong i = get_global_id(0); i < n; i += get_global_size(0)) {
            const float c = cosh(x_buffer[i]);
            y_buffer[i] = 1.0 / (c * c);
        }
    }
    "#;

#[test]
fn gemm() -> Result<()> {
    const M: usize = 5;
    const N: usize = 3;
    const K: usize = 2;

    let tanh = FloatTanh::new(None)?;
    std::hint::black_box(tanh);

    Ok(())
}
