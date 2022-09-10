#![feature(nonzero_min_max)]
use blaze_rs::{prelude::*, context::scope_async};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn invalid_raw () -> Result<()> {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;

    scope(|s| {
        let left = buffer.read(s, ..2, None)?.join();
        println!("{left:?}");
        return Ok(())
    })?;

    buffer.write_blocking(1, &[9, 8], None)?;
    println!("{buffer:?}");

    Ok(())
}

#[cfg(feature = "futures")]
#[tokio::test]
async fn test_async () {
    let mut buffer = Buffer::new(&[1, 2, 3, 4, 5], MemAccess::default(), false)?;

    let a = scope_async(|s| async {
        let left = buffer.read(s, ..2, None)?.join_async()?.await;
        println!("{left:?}");
        return Ok(())
    });

    buffer.write_blocking(1, &[9, 8], None)?;
    println!("{buffer:?}");

    Ok(())
}