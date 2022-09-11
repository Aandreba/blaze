use std::{time::{SystemTime}, mem::MaybeUninit};
use blaze_rs::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[inline(always)]
fn rng_code () -> String {
    let nanos = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    format!("#define TIME {}l\n{}", nanos.as_nanos(), include_str!("rng.cl"))
}

#[blaze(Rng)]
#[link = rng_code()]
pub extern "C" {
    fn next_bytes (n: u32, out: *mut MaybeUninit<u8>);
}

#[test]
fn main () -> Result<()> {
    const SIZE : usize = 50;

    let rng = Rng::new(None)?;
    let mut random = Buffer::<u8>::new_uninit(SIZE, MemAccess::WRITE_ONLY, false)?;
    
    let random = unsafe {
        let _ = rng.next_bytes_blocking(SIZE as u32, &mut random, [SIZE], None, None)?;
        random.assume_init()  
    };

    let _ = random.read_event(.., EMPTY)?.wait()?;
    Ok(())
}