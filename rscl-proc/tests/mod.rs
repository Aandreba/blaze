use rscl::{context::SimpleContext, buffer::{Buffer, WriteBuffer, RawBuffer}, event::Event};
use rscl_proc::{global_context, rscl};

#[rscl(Arith)]
#[link(include_str!("arith.cl"))]
pub extern "C" {
    #[link_name = "sum"]
    fn add (len: u64, lhs: *const f32, rhs: *const f32, out: *mut f32);
}

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn test () {
    let arith = Arith::new(None).unwrap();

    let lhs = Buffer::new(&[1f32, 2., 3., 4., 5.], false).unwrap();
    let rhs = Buffer::new(&[6f32, 7., 8., 9., 10.], false).unwrap();
    let mut out = unsafe { WriteBuffer::<f32>::uninit(5, false).unwrap() };

    let add = unsafe { arith.add(5, &lhs, &rhs, &mut out, [5, 1, 1], None, []).unwrap() };
    add.wait().unwrap();

    let out = out.read_all([]).unwrap().wait().unwrap();
    println!("{out:?}")
}