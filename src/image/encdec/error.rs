use core::ffi::c_char;
use std::{backtrace::Backtrace, sync::Arc, ffi::CString, fmt::Debug};
use ffmpeg_sys_next::*;

pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub desc: CString,
    #[cfg(debug_assertions)]
    pub backtrace: Arc<Backtrace>
}

impl Error {
    #[inline]
    pub fn new (kind: ErrorKind, desc: impl ToString) -> Error {
        Self::with_desc(kind, CString::new(desc.to_string()).unwrap())
    }

    #[inline]
    pub fn with_desc (kind: ErrorKind, desc: CString) -> Error {
        Self {
            kind,
            desc,
            #[cfg(debug_assertions)]
            backtrace: Arc::new(Backtrace::capture())
        }
    }

    #[inline(always)]
    pub fn from_kind (kind: ErrorKind) -> Error {
        Self::with_desc(kind, error_desc(kind as i32))
    }
}

impl Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(debug_assertions))]
        return write!(f, "{:?}: {:?}", &self.kind, self.desc);
        #[cfg(debug_assertions)]
        return write!(f, "{:?}: {:?}\n{}", &self.kind, &self.desc, &self.backtrace);
    }
}

impl From<i32> for Error {
    #[inline]
    fn from(x: i32) -> Self {
        let kind = match x {
            AVERROR_BSF_NOT_FOUND => ErrorKind::BsfNotFound,
            AVERROR_BUG => ErrorKind::Bug,
            AVERROR_BUFFER_TOO_SMALL => ErrorKind::BufferTooSmall,
            AVERROR_DECODER_NOT_FOUND => ErrorKind::DecoderNotFound,
            AVERROR_DEMUXER_NOT_FOUND => ErrorKind::DemuxerNotFound,
            AVERROR_ENCODER_NOT_FOUND => ErrorKind::EncoderNotFound,
            AVERROR_EOF => ErrorKind::Eof,
            AVERROR_EXIT => ErrorKind::Exit,
            AVERROR_EXTERNAL => ErrorKind::External,
            AVERROR_FILTER_NOT_FOUND => ErrorKind::FilterNotFound,
            AVERROR_INVALIDDATA => ErrorKind::InvalidData,
            AVERROR_MUXER_NOT_FOUND => ErrorKind::MuxerNotFound,
            AVERROR_OPTION_NOT_FOUND => ErrorKind::OptionNotFound,
            AVERROR_PATCHWELCOME => ErrorKind::Patchwelcome,
            AVERROR_PROTOCOL_NOT_FOUND => ErrorKind::ProtocolNotFound,
            AVERROR_STREAM_NOT_FOUND => ErrorKind::StreamNotFound,
            AVERROR_BUG2 => ErrorKind::Bug2,
            AVERROR_UNKNOWN => ErrorKind::Unknown,
            AVERROR_HTTP_BAD_REQUEST => ErrorKind::HttpBadRequest,
            AVERROR_HTTP_UNAUTHORIZED => ErrorKind::HttpUnauthorized,
            AVERROR_HTTP_FORBIDDEN => ErrorKind::HttpForbidden,
            AVERROR_HTTP_NOT_FOUND => ErrorKind::HttpNotFound,
            AVERROR_HTTP_OTHER_4XX => ErrorKind::HttpOther4xx,
            AVERROR_HTTP_SERVER_ERROR => ErrorKind::HttpServerError,
            other => {
                let desc = error_desc(other);
                return Self::with_desc(ErrorKind::Unknown, desc)
            }
        };

        Self::from_kind(kind)
    }
}

impl From<ErrorKind> for Error {
    #[inline(always)]
    fn from(kind: ErrorKind) -> Self {
        Self::from_kind(kind)
    }
}

impl Into<ErrorKind> for Error {
    #[inline(always)]
    fn into(self) -> ErrorKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum ErrorKind {
    BsfNotFound = AVERROR_BSF_NOT_FOUND,
    Bug = AVERROR_BUG,
    BufferTooSmall = AVERROR_BUFFER_TOO_SMALL,
    DecoderNotFound = AVERROR_DECODER_NOT_FOUND,
    DemuxerNotFound = AVERROR_DEMUXER_NOT_FOUND,
    EncoderNotFound = AVERROR_ENCODER_NOT_FOUND,
    Eof = AVERROR_EOF,
    Exit = AVERROR_EXIT,
    External = AVERROR_EXTERNAL,
    FilterNotFound = AVERROR_FILTER_NOT_FOUND,
    InvalidData = AVERROR_INVALIDDATA,
    MuxerNotFound = AVERROR_MUXER_NOT_FOUND,
    OptionNotFound = AVERROR_OPTION_NOT_FOUND,
    Patchwelcome = AVERROR_PATCHWELCOME,
    ProtocolNotFound = AVERROR_PROTOCOL_NOT_FOUND,
    StreamNotFound = AVERROR_STREAM_NOT_FOUND,
    Bug2 = AVERROR_BUG2,
    Unknown = AVERROR_UNKNOWN,
    HttpBadRequest = AVERROR_HTTP_BAD_REQUEST,
    HttpUnauthorized = AVERROR_HTTP_UNAUTHORIZED,
    HttpForbidden = AVERROR_HTTP_FORBIDDEN,
    HttpNotFound = AVERROR_HTTP_NOT_FOUND,
    HttpOther4xx = AVERROR_HTTP_OTHER_4XX,
    HttpServerError = AVERROR_HTTP_SERVER_ERROR
}

impl From<i32> for ErrorKind {
    #[inline]
    fn from(x: i32) -> Self {
        match x {
            AVERROR_BSF_NOT_FOUND => Self::BsfNotFound,
            AVERROR_BUG => Self::Bug,
            AVERROR_BUFFER_TOO_SMALL => Self::BufferTooSmall,
            AVERROR_DECODER_NOT_FOUND => Self::DecoderNotFound,
            AVERROR_DEMUXER_NOT_FOUND => Self::DemuxerNotFound,
            AVERROR_ENCODER_NOT_FOUND => Self::EncoderNotFound,
            AVERROR_EOF => Self::Eof,
            AVERROR_EXIT => Self::Exit,
            AVERROR_EXTERNAL => Self::External,
            AVERROR_FILTER_NOT_FOUND => Self::FilterNotFound,
            AVERROR_INVALIDDATA => Self::InvalidData,
            AVERROR_MUXER_NOT_FOUND => Self::MuxerNotFound,
            AVERROR_OPTION_NOT_FOUND => Self::OptionNotFound,
            AVERROR_PATCHWELCOME => Self::Patchwelcome,
            AVERROR_PROTOCOL_NOT_FOUND => Self::ProtocolNotFound,
            AVERROR_STREAM_NOT_FOUND => Self::StreamNotFound,
            AVERROR_BUG2 => Self::Bug2,
            AVERROR_UNKNOWN => Self::Unknown,
            AVERROR_HTTP_BAD_REQUEST => Self::HttpBadRequest,
            AVERROR_HTTP_UNAUTHORIZED => Self::HttpUnauthorized,
            AVERROR_HTTP_FORBIDDEN => Self::HttpForbidden,
            AVERROR_HTTP_NOT_FOUND => Self::HttpNotFound,
            AVERROR_HTTP_OTHER_4XX => Self::HttpOther4xx,
            AVERROR_HTTP_SERVER_ERROR => Self::HttpServerError,
            _ => Self::Unknown
        }
    }
}

fn error_desc (err: i32) -> CString {
    extern "C" {
        fn strlen (p: *const c_char) -> usize;
    }

    let mut str = Vec::<c_char>::with_capacity(AV_ERROR_MAX_STRING_SIZE);
    let desc = unsafe {
        av_strerror(err, str.as_mut_ptr(), AV_ERROR_MAX_STRING_SIZE)
    };

    match desc {
        x if x < 0 => unsafe {
            let len = strlen(str.as_ptr()) + 1;
            str.set_len(len);
            str.shrink_to_fit();

            let cstr = CString::from_raw(str.as_mut_ptr());
            core::mem::forget(str);
            cstr
        },

        _ => CString::new(Vec::new()).unwrap()
    }
}