#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Rule error {0}")]
    RuleError(String),
    #[error("Io error {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("Join error {0:?}")]
    JoinError(#[from] tokio::task::JoinError),
}

pub type Result<T> = std::result::Result<T, Error>;
