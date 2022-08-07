use std::{sync::Arc, backtrace::Backtrace, fmt::Debug, ffi::{c_char, c_int, CStr}, mem::MaybeUninit};
use ffmpeg_sys_next::{AV_ERROR_MAX_STRING_SIZE, av_strerror, AVERROR_EOF};

pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(Clone)]
pub struct Error {
    pub errnum: c_int,
    pub desc: Arc<str>,
    #[cfg(debug_assertions)]
    pub backtrace: Arc<Backtrace>,
}

impl Error {
    #[inline]
    pub fn new (errnum: c_int, desc: impl ToString) -> Self {
        Self {
            errnum,
            desc: Arc::<str>::from(desc.to_string()),
            #[cfg(debug_assertions)]
            backtrace: Arc::new(Backtrace::capture()),
        }
    }

    pub fn from_id (errnum: c_int) -> Self {
        let mut desc = MaybeUninit::<c_char>::uninit_array::<AV_ERROR_MAX_STRING_SIZE>();
        unsafe {
            av_strerror(errnum, desc.as_mut_ptr() as *mut c_char, AV_ERROR_MAX_STRING_SIZE);
        }

        let desc = unsafe {
            String::from_utf8_lossy(CStr::from_ptr(desc.as_ptr() as *const _).to_bytes())
        };

        let desc = Arc::<str>::from(desc);
        #[cfg(debug_assertions)]
        let backtrace = Arc::new(Backtrace::capture());

        Self { errnum, desc, #[cfg(debug_assertions)] backtrace }
    }

    #[inline(always)]
    pub fn try_from_id (errnum: c_int) -> Result<()> {
        if errnum >= 0 {
            return Ok(());
        }

        Err(Self::from_id(errnum))
    }

    #[inline(always)]
    pub const fn is_eof (&self) -> bool {
        self.errnum == AVERROR_EOF
    }
}

impl Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR {}: {:?}", self.errnum, self.desc);
        #[cfg(debug_assertions)]
        write!(f, "\n{}", self.backtrace);

        Ok(())
    }
}