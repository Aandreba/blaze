#![feature(nonzero_min_max)]

use blaze_rs::prelude::*;
use rand::random;
use tokio::spawn;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;
    let read = buffer.read(.., EMPTY)?.to_raw();
    // Problem. The status of read is unknown
    let write = buffer.write(2, vec![2], EMPTY)?;

    read.wait()?;

    Ok(())
}


#[tokio::test]
async fn sync () {
    const SIZE : usize = 100_000;
    let big_buffer : &'static mut [f32] = Vec::<f32>::with_capacity(SIZE).leak();

    let write = spawn(async {
        for v in big_buffer {
            *v = random();
        }
    });
    drop(write);

    let read = &big_buffer[..10];
}

async fn write_buffer (v: &'static mut [f32]) {
    spawn(async move {
        for v in v {
            *v = random();
        }
    }).await.unwrap()
}

async fn read_buffer (v: &'static [f32]) {
    todo!()
}