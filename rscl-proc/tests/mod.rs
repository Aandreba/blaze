use rscl::{context::SimpleContext};
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
}