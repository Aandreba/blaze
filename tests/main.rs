use rscl::{core::*, context::{SimpleContext, Global}};
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
    println!("{}", core::mem::size_of::<Device>());
    println!("{}", core::mem::size_of::<Option<Device>>());

    println!("{}", core::mem::size_of::<bool>());
    println!("{}", core::mem::size_of::<Option<bool>>());

    let dev = Device::first().unwrap();
    println!("{:?}", Global.num_devices());
    Ok(())
}

#[cfg(feature = "image")]
#[test]
fn flag () {
    use image::{Rgba, io::Reader, imageops::{resize, FilterType}};
    use rscl::{image::Image2D, buffer::flags::MemAccess};

    let img2 = Reader::open("tests/test2.jpg").unwrap()
        .decode().unwrap()
        .into_rgba8();

    let mut img1 = Image2D::<Rgba<u8>>::from_file("tests/test.png", MemAccess::default(), false).unwrap();
    let img2 = resize(&img2, 64, 64, FilterType::Gaussian);

    // todo test write
    image.save("tests/test_slice.png").unwrap();
}