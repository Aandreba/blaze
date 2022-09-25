use blaze_rs::{prelude::*, buffer};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[test]
fn read () -> Result<()> {
    let buf = buffer![1, 2, 3, 4, 5]?;
    let blocking = buf.read_blocking(2.., None)?;
    let scope = scope(|s| buf.read(s, ..=3, None)?.join())?;

    assert_eq!(blocking, vec![3, 4, 5]);
    assert_eq!(scope, vec![1, 2, 3, 4]);

    Ok(())
}

#[test]
fn write () -> Result<()> {
    let mut buf = buffer![1, 2, 3, 4, 5]?;
    buf.write_blocking(0, &[6, 7], None)?;
    scope(|s| buf.write(s, 2, &[8, 9], None)?.join())?;

    assert_eq!(buf, buffer![6, 7, 8, 9, 5]?);
    Ok(())
}

/* RECT */
cfg_if::cfg_if! {
    if #[cfg(feature = "cl1_1")] {
        #[test]
        fn read_rect () -> Result<()> {
            let buf = rect_buf()?;
            
            let blocking = buf.read_blocking((.., 1..), None)?;
            let scope = scope(|s| buf.read(s, (1.., ..=1), None)?.join())?;

            assert_eq!(blocking.as_slice(), &[4, 5, 6, 7, 8, 9]);
            assert_eq!(scope.as_slice(), &[2, 3, 5, 6]);

            Ok(())
        }

        #[inline(always)]
        fn rect_buf () -> Result<RectBuffer2D<i32>> {
            RectBuffer2D::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3, MemAccess::default(), false)
        }
    }
}