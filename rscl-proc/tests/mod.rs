#![feature(allocator_api, new_uninit)]

use core::f32;
use std::{f32::consts::{PI, E, TAU}, mem::MaybeUninit};

use rscl::{context::SimpleContext, prelude::{Result, Event}, event::WaitList, buffer::{Buffer, flags::MemAccess}, svm::{SvmVecExt, SvmVec, Svm, SvmBox}};
use rscl_proc::{global_context, rscl};

#[rscl(Arith)]
#[link(include_str!("arith.cl"))]
pub extern "C" {
    #[link_name = "sum"]
    fn add (len: u64, lhs: *const f32, rhs: *const f32, out: *mut MaybeUninit<f32>);
}

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn test () -> Result<()> {
    let arith = Arith::new(None)?;
    let lhs = Buffer::new(&[1., 2., 3., 4., 5.], MemAccess::default(), false)?;
    let rhs = Buffer::new(&[PI, E, TAU, 2., -1.], MemAccess::default(), false)?;
    let mut out = SvmBox::new_uninit_slice_in(5, Svm::new());
    out[0].write(2.);
    println!("{out:?}");

    let evt = unsafe { arith.add(5, &lhs, &rhs, &mut out, [5, 1, 1], None, WaitList::EMPTY)? };
    let (_, duration) = evt.wait_with_duration()?;
    let out = unsafe { out.assume_init() };

    // todo fix svm memory not writing

    println!("{out:?}: {duration:?}");
    Ok(())
}