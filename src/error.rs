#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CliftError")]
    CliftError,

    #[error("UploadError: {}", _0)]
    UploadError(#[from] clift::commands::UploadError),

    #[error("UploadError: {}", _0)]
    ReqwestError(#[from] reqwest::Error),

    #[error("URLParseError: {}", _0)]
    UrlParseError(#[from] url::ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
