use rscl::{context::SimpleContext, buffer::{Buffer, WriteBuffer, MemObject}, event::Event};
use rscl_proc::{global_context, rscl_c};

rscl_c! {
    pub struct Arith {
        kernel void add (const ulong n, __global const float* inn, __global const float* rhs, __global float* out) {
            for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
                out[id] = inn[id] + rhs[id];
            }
        }
    }
}

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn test () {
    let arith = Arith::new(None).unwrap();

    let lhs = Buffer::new(&[1f32, 2., 3., 4., 5.], false).unwrap();
    let rhs = Buffer::new(&[6f32, 7., 8., 9., 10.], false).unwrap();
    let mut out = unsafe { WriteBuffer::<f32>::uninit(5, false).unwrap() };

    let add = arith.add(5, &lhs, &rhs, &mut out, [5, 1, 1], None, []).unwrap();
    add.wait().unwrap();

    let out = out.read_all([]).unwrap().wait().unwrap();
    println!("{out:?}")
}