use rscl::{core::*, context::{SimpleContext}, buffer::{Buffer, flags::MemAccess, BufferRect2D, rect::Rect2D}, event::{WaitList, FlagEvent}, prelude::{Event, Global, Context}, image::{Image2D, Sampler, SamplerProperties, AddressingMode}};
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
    let mut rect = Rect2D::<f32>::new(&[1.0, 2., 3., 4., 5., 6., 7., 8., 9.], 3).unwrap(); // 3 x 3
    let buf = BufferRect2D::new(&rect, MemAccess::default(), false).unwrap();
    buf.read_into([1, 0], &mut rect, [0, 0], [2, 2], WaitList::EMPTY).unwrap().wait_unwrap();
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