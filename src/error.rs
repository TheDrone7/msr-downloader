use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("API error: {message}")]
    Api { message: String },

    #[error("Download failed: {0}")]
    Download(String),

    #[error("File operation failed: {0}")]
    File(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type Result<T> = std::result::Result<T, Error>;
