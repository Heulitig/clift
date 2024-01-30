#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CliftError")]
    CliftError,
}

pub type Result<T> = std::result::Result<T, Error>;
