use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReflectError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("evaluation error: {0}")]
    Eval(String),
    #[error("pattern error: {0}")]
    Pattern(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, ReflectError>;
