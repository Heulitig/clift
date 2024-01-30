#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("UploadError: {}", _0)]
    UploadError(#[from] clift::commands::UploadError),
}

pub type Result<T> = std::result::Result<T, Error>;
