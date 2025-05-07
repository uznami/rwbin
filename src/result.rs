use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("io error: {0}")]
    IoError(std::io::Error),

    #[error("not enough bytes: expected {expected}, actual: {actual}")]
    NotEnoughBytes { expected: usize, actual: usize },

    #[error("invalid argument")]
    InvalidArgument,

    #[error("invalid data format: {0}")]
    InvalidDataFormat(String),
}

impl ReadError {
    pub fn not_enough_bytes(expected: usize, actual: usize) -> Self {
        Self::NotEnoughBytes { expected, actual }
    }
    pub fn invalid_data_format<M: AsRef<str>>(msg: M) -> Self {
        Self::InvalidDataFormat(msg.as_ref().to_string())
    }
    pub fn io_error(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

#[derive(Error, Debug)]
pub enum WriteError {
    #[error("io error: {0}")]
    IoError(std::io::Error),
}
impl WriteError {
    pub fn io_error(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

pub type ReadResult<T> = Result<T, ReadError>;
pub type WriteResult<T> = Result<T, WriteError>;
