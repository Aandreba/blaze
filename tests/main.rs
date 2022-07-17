use std::sync::Arc;

use rscl::{core::*, context::{SimpleContext}, buffer::{flags::MemAccess, Buffer, events::ReadBuffer}, event::{WaitList}, prelude::{Global, Context, EventExt, Event}};
use rscl_proc::{global_context};

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
    let buffer = Arc::new(Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?);
    let one = buffer.clone().read_owned(..2, WaitList::EMPTY)?;
    let two = buffer.read_owned(2.., WaitList::EMPTY)?;

    let join = ReadBuffer::join([one, two])?;
    let out = join.wait_with_time()?;

    println!("{out:?}");
    Ok(())
}