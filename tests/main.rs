use tokio::test;
use std::{time::Duration, sync::Arc};
use rscl::{macros::global_context, core::*, context::{SimpleContext}, event::FlagEvent, buffer::{Buffer, flags::MemFlags}};

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[global_context]
pub static CONTEXT : SimpleContext = SimpleContext::new(Device::first().unwrap()).unwrap();

#[test]
async fn test () -> Result<()> {
    let buffer = Buffer::new(&[1, 2, 3, 4, 5], MemFlags::default())?;
    let read = buffer.read(..)?;

    Ok(())
}