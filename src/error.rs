use thiserror::Error;

#[derive(Error, Debug)]
pub enum PortlyError {
    #[error("Port {port} is not in use")]
    PortNotFound {
        port: u16,
        suggestion: Option<String>,
    },

    #[error("Process {pid} not found")]
    ProcessNotFound {
        pid: u32,
        suggestion: Option<String>,
    },

    #[error("Platform operation failed: {0}")]
    PlatformError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl PortlyError {
    /// Returns the suggestion for this error, if available.
    #[must_use]
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Self::PortNotFound { suggestion, .. } | Self::ProcessNotFound { suggestion, .. } => {
                suggestion.as_deref()
            }
            Self::PlatformError(_) | Self::IoError(_) => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, PortlyError>;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_not_found_without_suggestion() {
        let error = PortlyError::PortNotFound {
            port: 8080,
            suggestion: None,
        };
        assert_eq!(error.to_string(), "Port 8080 is not in use");
        assert_eq!(error.suggestion(), None);
    }

    #[test]
    fn test_port_not_found_with_suggestion() {
        let error = PortlyError::PortNotFound {
            port: 8080,
            suggestion: Some("Try running 'portly list' to see active ports".to_string()),
        };
        assert_eq!(error.to_string(), "Port 8080 is not in use");
        assert_eq!(
            error.suggestion(),
            Some("Try running 'portly list' to see active ports")
        );
    }

    #[test]
    fn test_process_not_found_without_suggestion() {
        let error = PortlyError::ProcessNotFound {
            pid: 1234,
            suggestion: None,
        };
        assert_eq!(error.to_string(), "Process 1234 not found");
        assert_eq!(error.suggestion(), None);
    }

    #[test]
    fn test_process_not_found_with_suggestion() {
        let error = PortlyError::ProcessNotFound {
            pid: 1234,
            suggestion: Some("The process may have already exited".to_string()),
        };
        assert_eq!(error.to_string(), "Process 1234 not found");
        assert_eq!(
            error.suggestion(),
            Some("The process may have already exited")
        );
    }

    #[test]
    fn test_platform_error_has_no_suggestion() {
        let error = PortlyError::PlatformError("System call failed".to_string());
        assert_eq!(error.suggestion(), None);
    }

    #[test]
    fn test_io_error_has_no_suggestion() {
        let error = PortlyError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert_eq!(error.suggestion(), None);
    }
}
