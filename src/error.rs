use thiserror::Error;

#[derive(Error, Debug)]
pub enum PortlyError {
    #[error("Port {0} is not in use")]
    PortNotFound(u16),

    #[error("Process {0} not found")]
    ProcessNotFound(u32),

    #[error("Platform operation failed: {0}")]
    PlatformError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PortlyError>;
