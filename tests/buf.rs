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

#[cfg(feature = "cl1_1")]
#[test]
fn slice () -> Result<()> {
    let buf = buffer![1, 2, 3, 4, 5]?;
    let slice = buf.slice(1..)?;
    let slice2 = slice.slice(..2)?;

    println!("{buf:?}, {slice:?}, {slice2:?}");
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

        #[test]
        fn write_rect () -> Result<()> {
            let mut buf = rect_buf()?; 
            let host = Rect2D::new(&[10, 11, 12, 13], 2);
            
            buf.write_blocking(None, host.as_parts(), [0, 1], None, None)?;
            scope(|s| buf.write(s, [1, 1], host.as_parts(), None, None, None).map(|_| ()))?;

            assert_eq!(buf.map_blocking(.., None)?.deref(), &[12, 13, 3, 4, 10, 11, 7, 12, 13]);
            Ok(())
        }

        #[test]
        fn copy_rect () -> Result<()> {
            let mut buf = rect_buf()?; // 3 x 3
            let buf2 = RectBuffer2D::new(&[10, 11, 12, 13, 14, 15], 2, MemAccess::default(), false)?; // 2 x 3

            println!("{buf:?}, {buf2:?}");
            //buf.copy_from_blocking(None, &buf2, None, None, None)?;
            scope(|s| buf.copy_from(s, [1, 1], &buf2, [0, 1], None, None).map(|_| ()))?;

            println!("{buf:?}");
            Ok(())
        }

        #[inline(always)]
        fn rect_buf () -> Result<RectBuffer2D<i32>> {
            RectBuffer2D::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3, MemAccess::default(), false)
        }
    }
}