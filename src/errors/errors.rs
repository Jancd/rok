#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("hyper error")]
    HyperError(#[from] hyper::Error),
    #[error("std io error")]
    IOError(#[from] std::io::Error),
    #[error("http error")]
    HttpError(#[from] hyper::http::Error),
    #[error("serde_json error")]
    JsonError(#[from] serde_json::Error),
    #[error("decode query string error")]
    QueryError(#[from] serde_urlencoded::de::Error),
    #[error("internal error")]
    Message(String),
    #[error("invalid request header {msg:?}")]
    InvalidHeader { msg: String },
    #[error("missing AppState {msg:?}")]
    MissingAppState { msg: String },
    #[error("missing url param {msg:?}")]
    MissingParam { msg: String },
    #[error("missing cookie {msg:?}")]
    MissingCookie { msg: String },
    #[error("missing header {msg:?}")]
    MissingHeader { msg: String },
    #[error("invalid param {msg:?} as {expected:?}, {err:?}")]
    InvalidParam {
        err: String,
        msg: String,
        expected: &'static str,
    },
}

impl<'a> From<&'a str> for Error {
    fn from(s: &'a str) -> Self {
        Error::Message(s.to_string())
    }
}


impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Message(s)
    }
}

pub fn invalid_params(
    err: impl std::error::Error,
    msg: impl ToString,
    expected: &'static str,
) -> Error {
    Error::InvalidParam {
        err: err.to_string(),
        msg: msg.to_string(),
        expected,
    }
}

pub fn invalid_header(msg: impl ToString) -> Error {
    Error::InvalidHeader { msg: msg.to_string() }
}

pub fn missing_app_state(msg: impl ToString) -> Error {
    Error::MissingAppState { msg: msg.to_string() }
}

pub fn missing_param(msg: impl ToString) -> Error {
    Error::MissingParam { msg: msg.to_string() }
}

pub fn missing_cookie(msg: impl ToString) -> Error {
    Error::MissingCookie { msg: msg.to_string() }
}

pub fn missing_header(msg: impl ToString) -> Error {
    Error::MissingHeader { msg: msg.to_string() }
}

#[macro_export]
macro_rules! error_message {
    ($message:literal) => {
        $crate::Error::Message($message.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Error::Message(format!($fmt, $($arg)*))
    };
}