use std::sync::Mutex;
use rscl::{core::*, context::{SimpleContext}, buffer::{flags::MemAccess, BufferRect2D, rect::Rect2D, Buffer}, event::{WaitList}, prelude::{Event, Global, Context}};
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
fn flag () -> Result<()> {
    let mut buffer = Buffer::<f32>::new_zeroed(5, MemAccess::default(), false)?;
    let (map, duration) = buffer.map_all_mut(WaitList::EMPTY)?.wait_with_duration()?;
    println!("{map:?}: {duration:?}");
    Ok(())
}