use rscl::{macros::global_context, core::*, context::{SimpleContext}, buffer::{Buffer, flags::{MemFlags, MemAccess}}, event::{Event, WaitList}};

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[global_context]
pub static CONTEXT : SimpleContext = SimpleContext::new(Device::first().unwrap()).unwrap();

#[test]
fn test () -> Result<()> {
    let (_, kernels) = Program::from_source(PROGRAM)?;

    let alpha = Buffer::new(&[1f32, 2., 3., 4., 5.], MemFlags::default())?;
    let beta = Buffer::new(&[1f32, 2., 3., 4., 5.], MemFlags::default())?;
    let gamma = unsafe { Buffer::<f32>::uninit(5, MemAccess::WRITE_ONLY)? };

    let kernel = &kernels[0];
    let evt = kernel.build([5, 1, 1])?
        .set_value(0, 5u64)
        .set_mem_buffer(1, &beta)
        .set_mem_buffer(2, &alpha)
        .set_mem_buffer(3, &gamma)
        .build()?;

    // todo fix waiting events
    //let wait = WaitList::from_boxed_slice(Box::new([evt.as_ref().id()]));
    let gamma = gamma.read_all([evt.raw()])?.wait()?;
    println!("{gamma:?}");

    Ok(())
}