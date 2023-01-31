use blaze_rs::{prelude::*, buffer};

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

#[cfg(feature = "cl1_1")]
#[test]
fn read () -> Result<()> {
    use rand::seq::SliceRandom;

    let buf = buffer![1, 2, 3, 4, 5]?;
    //let blocking = buf.read_blocking(2.., None)?;
    
    scope(|s| {
        let evt = buf.read(s, ..=3, None)?;
        let cb = evt.then_scoped(s, |mut x| {
            x.shuffle(&mut rand::thread_rng());
            x
        })?;

        let v = cb.join_unwrap()?;
        println!("{v:?}");

        Ok(())
    })?;

    Ok(())
}

#[cfg(feature = "cl1_1")]
#[test]
fn cb () -> Result<()> {
    use blaze_rs::event::FlagEvent;

    let flag = FlagEvent::new()?;
    let handle = flag.subscribe().on_complete(|_, _| println!("Done!"))?;
    assert!(flag.try_mark(None)?);
    handle.join_unwrap();

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
    Ok(())
}

// expect panic
#[cfg(feature = "cl1_1")]
#[should_panic]
#[test]
fn double_slice () {
    let buf = buffer![1, 2, 3, 4, 5].unwrap();
    let slice = buf.slice(1..).unwrap();
    let slice2 = slice.slice(..2).unwrap();
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

        /*#[test]
        fn write_rect () -> Result<()> {
            let mut buf = rect_buf()?; 
            let host = Rect2D::new(&[10, 11, 12, 13], 2);
            
            buf.write_blocking(None, host.as_parts(), [0, 1], None, None)?;
            scope(|s| buf.write(s, [1, 1], host.as_parts(), None, None, None).map(|_| ()))?;

            assert_eq!(buf.map_blocking(.., None)?.deref(), &[12, 13, 3, 4, 10, 11, 7, 12, 13]);
            Ok(())
        }*/

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