use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlomError {
    #[error("unsupported input: {0}")]
    UnsupportedInput(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("api error: {0}")]
    Api(String),
    #[error("parse error: {0}")]
    Parse(String),
}

pub type FlomResult<T> = Result<T, FlomError>;
