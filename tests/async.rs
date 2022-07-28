use blaze::{prelude::*, context::SimpleContext};
use tokio::select;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[tokio::test]
async fn test () -> Result<()> {
    let mut big_buffer = Buffer::<i32>::new_uninit(u16::MAX as usize, MemAccess::READ_WRITE, false)?;
    let evt = big_buffer.fill_init(2, .., EMPTY)?;
    let fut = evt.wait_async()?;

    let checker = tokio::spawn(async move {
        loop {
            println!("Still waiting");
        }
    });

    select! {
        _ = checker => panic!("Something has gone wrong!"),
        _ = fut => println!("Done")
    };

    Ok(())
}