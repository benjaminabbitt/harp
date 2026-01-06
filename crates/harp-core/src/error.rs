use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("division by zero")]
    DivisionByZero,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
