use thiserror::Error;

#[derive(Error, Debug)]
pub enum PortlyError {
    #[error("Port {0} is not in use")]
    PortNotFound(u16),

    #[error("Process {0} not found")]
    ProcessNotFound(u32),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Platform operation failed: {0}")]
    PlatformError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PortlyError>;
