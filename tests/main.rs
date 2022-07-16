use std::sync::Mutex;

use rscl::{core::*, context::{SimpleContext}, buffer::{flags::MemAccess, BufferRect2D, rect::Rect2D}, event::{WaitList}, prelude::{Event, Global, Context}};
use rscl_proc::global_context;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[test]
fn program () -> Result<()> {
    let version = Global.next_queue().size()?;
    println!("{version:?}");
    Ok(())
}

#[test]
fn flag () {
    /*
        [
            [1, 2, 3],
            [4, 5, 6],
            [7, 8, 9]
        ]
    */
    let mut rect = Mutex::new(Rect2D::<f32>::new(&[1., 2., 3., 4., 5., 6., 7., 8., 9.], 3).unwrap()); // 3 x 3
    let rect = rect.lock().unwrap();

    let buf = BufferRect2D::new(&rect, MemAccess::default(), false).unwrap();
    let (buf, rect) = buf.read_into([1, 0], rect, [0, 0], [2, 2], WaitList::EMPTY).unwrap().wait_unwrap();
    
    // TODO FIX

    /*
        [
            [2, 3, 3],
            [5, 6, 6],
            [7, 8, 9]
        ]
    */

    println!("{rect:?}")
}