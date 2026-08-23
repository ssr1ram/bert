// Error types for bert CLI
use thiserror::Error;

/// Main error type for bert CLI operations
#[derive(Error, Debug)]
pub enum BertError {
    /// Configuration file errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Task file not found
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// File system errors
    #[error("File error: {0}")]
    FileError(String),

    /// Invalid input from user
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Parent task not found when creating subtask
    #[error("Parent task {0} not found. Cannot create subtask.")]
    ParentNotFound(String),

    /// Generic not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Resource already exists
    #[error("Already exists: {0}")]
    AlreadyExists(String),
}

/// Result type alias for bert operations
pub type Result<T> = std::result::Result<T, BertError>;

/// Exit codes for the CLI
pub mod exit_codes {
    /// Configuration error
    pub const CONFIG_ERROR: i32 = 2;

    /// Task not found
    pub const TASK_NOT_FOUND: i32 = 3;

    /// File error
    pub const FILE_ERROR: i32 = 4;

    /// Invalid input
    pub const INVALID_INPUT: i32 = 5;
}

impl BertError {
    /// Convert error to appropriate exit code
    pub fn exit_code(&self) -> i32 {
        match self {
            BertError::ConfigError(_) => exit_codes::CONFIG_ERROR,
            BertError::TaskNotFound(_) | BertError::ParentNotFound(_) | BertError::NotFound(_) => {
                exit_codes::TASK_NOT_FOUND
            }
            BertError::FileError(_) | BertError::IoError(_) | BertError::AlreadyExists(_) => {
                exit_codes::FILE_ERROR
            }
            BertError::InvalidInput(_) => exit_codes::INVALID_INPUT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error() {
        let error = BertError::ConfigError("missing field".to_string());
        assert!(error.to_string().contains("missing field"));
        assert_eq!(error.exit_code(), exit_codes::CONFIG_ERROR);
    }

    #[test]
    fn test_task_not_found_error() {
        let error = BertError::TaskNotFound("task-08".to_string());
        assert!(error.to_string().contains("task-08"));
        assert_eq!(error.exit_code(), exit_codes::TASK_NOT_FOUND);
    }

    #[test]
    fn test_invalid_input_error() {
        let error = BertError::InvalidInput("bad format".to_string());
        assert!(error.to_string().contains("bad format"));
        assert_eq!(error.exit_code(), exit_codes::INVALID_INPUT);
    }

    #[test]
    fn test_parent_not_found_error() {
        let error = BertError::ParentNotFound("08".to_string());
        assert!(error.to_string().contains("Parent task 08 not found"));
        assert_eq!(error.exit_code(), exit_codes::TASK_NOT_FOUND);
    }
}
