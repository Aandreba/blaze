use rscl::{macros::global_context, core::{Device, Program}, context::{SingleContext}};

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[global_context]
pub static CONTEXT : SingleContext = SingleContext::new(Device::first().unwrap()).unwrap();

#[test]
fn test () {
    let prog = Program::from_source(PROGRAM).unwrap();
    println!("{}", prog.device_count().unwrap());
}