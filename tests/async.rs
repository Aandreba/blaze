use std::time::Duration;
use blaze::prelude::*;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[tokio::test]
async fn test () -> Result<()> {
    const SIZE : usize = u16::MAX as usize;
    let values = vec![1234; SIZE];

    let mut big_buffer = Buffer::<i32>::new_uninit(SIZE, MemAccess::READ_WRITE, false)?;
    let flag = FlagEvent::new()?;

    let read = big_buffer.write_init(0, &values, &flag)?;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        flag.complete(None)
    });

    let _ = read.wait_async()?.await;

    Ok(())
}