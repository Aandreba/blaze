use std::{path::Path, ffi::{CString, NulError}};

macro_rules! ffmpeg_tri {
    ($e:expr) => {{
        let err = $e;
        if err < 0 {
            return Err(super::error::Error::from(err))
        }
    }};

    ($($e:expr);+) => {{
        let mut err = 0;
        
        $(
            err = $e;
            if err < 0 {
                return Err(super::error::Error::from(err))
            }
        )+
    }};
}

macro_rules! ffmpeg_tri_panic {
    ($e:expr) => {{
        let err = $e;
        if err < 0 {
            panic!("{:?}", super::error::Error::from(err));
        }
    }};

    ($($e:expr);+) => {{
        let mut err = 0;
        
        $(
            err = $e;
            if err < 0 {
                panic!("{:?}", super::error::Error::from(err));
            }
        )+
    }};
}

mod error;
pub mod pixel;
mod stream;
mod alloc;

#[inline]
pub(super) fn path_str (path: impl AsRef<Path>) -> Result<CString, NulError> {
    let path = path.as_ref().to_string_lossy().into_owned();
    CString::new(path)
}