//! Unified error types for CareerBench
//! 
//! This module provides standardized error types across all modules,
//! ensuring consistent error handling and better error messages.

use std::fmt;

/// Main error type for the application
/// 
/// This enum represents all possible errors that can occur in CareerBench.
/// It implements `std::error::Error` for proper error handling and can be
/// converted to user-friendly messages.
#[derive(Debug, Clone)]
pub enum CareerBenchError {
    /// Database-related errors
    Database(DatabaseError),
    /// AI provider errors
    AiProvider(AiProviderError),
    /// Validation errors
    Validation(ValidationError),
    /// Configuration errors
    Configuration(ConfigurationError),
    /// File system errors
    FileSystem(FileSystemError),
    /// General application errors
    Application(String),
}

/// Database-specific errors
#[derive(Debug, Clone)]
pub enum DatabaseError {
    /// Connection failed
    ConnectionFailed(String),
    /// Query execution failed
    QueryFailed(String),
    /// Migration failed
    MigrationFailed(String),
    /// Constraint violation (e.g., unique constraint)
    ConstraintViolation(String),
    /// Record not found
    NotFound(String),
    /// Invalid data format
    InvalidData(String),
}

/// AI provider errors (re-exported from ai::errors)
pub use crate::ai::errors::AiProviderError;

/// Validation errors
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Required field is missing
    MissingField(String),
    /// Field has invalid format
    InvalidFormat(String),
    /// Field value is out of range
    OutOfRange(String),
    /// Field value violates business rule
    BusinessRule(String),
    /// General validation error
    General(String),
}

/// Configuration errors
#[derive(Debug, Clone)]
pub enum ConfigurationError {
    /// Setting not found
    SettingNotFound(String),
    /// Invalid setting value
    InvalidValue(String),
    /// Configuration file not found
    FileNotFound(String),
    /// Failed to parse configuration
    ParseError(String),
    /// Other configuration error
    Other(String),
}

/// File system errors
#[derive(Debug, Clone)]
pub enum FileSystemError {
    /// File not found
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Disk full
    DiskFull(String),
    /// General I/O error
    IoError(String),
}

impl fmt::Display for CareerBenchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CareerBenchError::Database(e) => write!(f, "Database error: {}", e),
            CareerBenchError::AiProvider(e) => write!(f, "AI error: {}", e),
            CareerBenchError::Validation(e) => write!(f, "Validation error: {}", e),
            CareerBenchError::Configuration(e) => write!(f, "Configuration error: {}", e),
            CareerBenchError::FileSystem(e) => write!(f, "File system error: {}", e),
            CareerBenchError::Application(msg) => write!(f, "Application error: {}", msg),
        }
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionFailed(msg) => write!(f, "Database connection failed: {}", msg),
            DatabaseError::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
            DatabaseError::MigrationFailed(msg) => write!(f, "Migration failed: {}", msg),
            DatabaseError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            DatabaseError::NotFound(msg) => write!(f, "Record not found: {}", msg),
            DatabaseError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ValidationError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ValidationError::OutOfRange(msg) => write!(f, "Value out of range: {}", msg),
            ValidationError::BusinessRule(msg) => write!(f, "Business rule violation: {}", msg),
            ValidationError::General(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigurationError::SettingNotFound(setting) => write!(f, "Setting not found: {}", setting),
            ConfigurationError::InvalidValue(msg) => write!(f, "Invalid configuration value: {}", msg),
            ConfigurationError::FileNotFound(path) => write!(f, "Configuration file not found: {}", path),
            ConfigurationError::ParseError(msg) => write!(f, "Failed to parse configuration: {}", msg),
            ConfigurationError::Other(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystemError::NotFound(path) => write!(f, "File not found: {}", path),
            FileSystemError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            FileSystemError::DiskFull(msg) => write!(f, "Disk full: {}", msg),
            FileSystemError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for CareerBenchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CareerBenchError::AiProvider(e) => Some(e),
            _ => None,
        }
    }
}

impl std::error::Error for DatabaseError {}
impl std::error::Error for ValidationError {}
impl std::error::Error for ConfigurationError {}
impl std::error::Error for FileSystemError {}

// Convenience conversions

impl From<DatabaseError> for CareerBenchError {
    fn from(err: DatabaseError) -> Self {
        CareerBenchError::Database(err)
    }
}

impl From<AiProviderError> for CareerBenchError {
    fn from(err: AiProviderError) -> Self {
        CareerBenchError::AiProvider(err)
    }
}

impl From<ValidationError> for CareerBenchError {
    fn from(err: ValidationError) -> Self {
        CareerBenchError::Validation(err)
    }
}

impl From<ConfigurationError> for CareerBenchError {
    fn from(err: ConfigurationError) -> Self {
        CareerBenchError::Configuration(err)
    }
}

impl From<FileSystemError> for CareerBenchError {
    fn from(err: FileSystemError) -> Self {
        CareerBenchError::FileSystem(err)
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::SqliteFailure(e, msg) => {
                let error_msg = msg.unwrap_or_else(|| format!("SQLite error code: {:?}", e.code));
                match e.code {
                    rusqlite::ErrorCode::ConstraintViolation => {
                        DatabaseError::ConstraintViolation(error_msg)
                    }
                    _ => DatabaseError::QueryFailed(error_msg),
                }
            }
            rusqlite::Error::QueryReturnedNoRows => {
                DatabaseError::NotFound("Query returned no rows".to_string())
            }
            _ => DatabaseError::QueryFailed(err.to_string()),
        }
    }
}

impl From<rusqlite::Error> for CareerBenchError {
    fn from(err: rusqlite::Error) -> Self {
        CareerBenchError::Database(err.into())
    }
}

impl From<std::io::Error> for FileSystemError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                FileSystemError::NotFound(err.to_string())
            }
            std::io::ErrorKind::PermissionDenied => {
                FileSystemError::PermissionDenied(err.to_string())
            }
            _ => FileSystemError::IoError(err.to_string()),
        }
    }
}

impl From<std::io::Error> for CareerBenchError {
    fn from(err: std::io::Error) -> Self {
        CareerBenchError::FileSystem(err.into())
    }
}

/// Convert CareerBenchError to user-friendly string for frontend
/// 
/// This function provides a simplified error message suitable for display
/// to users, hiding technical details while providing actionable information.
pub fn to_user_message(error: &CareerBenchError) -> String {
    match error {
        CareerBenchError::Database(e) => match e {
            DatabaseError::ConnectionFailed(_) => {
                "Failed to connect to database. Please try again.".to_string()
            }
            DatabaseError::QueryFailed(msg) => {
                format!("Database operation failed: {}", msg)
            }
            DatabaseError::MigrationFailed(_) => {
                "Database migration failed. Please check logs.".to_string()
            }
            DatabaseError::ConstraintViolation(msg) => {
                format!("Data conflict: {}", msg)
            }
            DatabaseError::NotFound(msg) => {
                format!("Record not found: {}", msg)
            }
            DatabaseError::InvalidData(msg) => {
                format!("Invalid data: {}", msg)
            }
        },
        CareerBenchError::AiProvider(e) => {
            // Use the existing error message utility
            crate::ai::error_messages::get_short_error_message(e)
        }
        CareerBenchError::Validation(e) => match e {
            ValidationError::MissingField(field) => {
                format!("Please fill in the required field: {}", field)
            }
            ValidationError::InvalidFormat(msg) => {
                format!("Invalid format: {}", msg)
            }
            ValidationError::OutOfRange(msg) => {
                format!("Value out of range: {}", msg)
            }
            ValidationError::BusinessRule(msg) => {
                format!("Validation failed: {}", msg)
            }
            ValidationError::General(msg) => {
                format!("Validation error: {}", msg)
            }
        },
        CareerBenchError::Configuration(e) => match e {
            ConfigurationError::SettingNotFound(setting) => {
                format!("Setting '{}' not found. Please configure it in Settings.", setting)
            }
            ConfigurationError::InvalidValue(msg) => {
                format!("Invalid configuration: {}", msg)
            }
            ConfigurationError::FileNotFound(_) => {
                "Configuration file not found. Using defaults.".to_string()
            }
            ConfigurationError::ParseError(msg) => {
                format!("Failed to parse configuration: {}", msg)
            }
            ConfigurationError::Other(msg) => {
                format!("Configuration error: {}", msg)
            }
        },
        CareerBenchError::FileSystem(e) => match e {
            FileSystemError::NotFound(path) => {
                format!("File not found: {}", path)
            }
            FileSystemError::PermissionDenied(path) => {
                format!("Permission denied: {}", path)
            }
            FileSystemError::DiskFull(_) => {
                "Disk is full. Please free up space.".to_string()
            }
            FileSystemError::IoError(msg) => {
                format!("File system error: {}", msg)
            }
        },
        CareerBenchError::Application(msg) => msg.clone(),
    }
}

/// Convert CareerBenchError to String for Tauri command return types
/// 
/// Tauri commands must return `Result<T, String>`, so this function
/// converts our structured error types to user-friendly strings.
/// This also automatically logs the error and records metrics before converting.
impl CareerBenchError {
    pub fn to_string_for_tauri(&self) -> String {
        // Log the error and record metrics before converting to string
        crate::logging::log_careerbench_error("TauriCommand", self);
        crate::error_logging::record_error_metric(self, "TauriCommand");
        to_user_message(self)
    }
    
    /// Log the error and return it (for use in error chains)
    pub fn log_and_return(self, context: &str) -> Self {
        crate::logging::log_careerbench_error(context, &self);
        crate::error_logging::record_error_metric(&self, context);
        self
    }
}

impl From<CareerBenchError> for String {
    fn from(err: CareerBenchError) -> String {
        err.to_string_for_tauri()
    }
}

/// Get a short error message for display in UI
/// 
/// Returns a concise error message suitable for toast notifications
/// or inline error displays.
pub fn get_short_error_message(error: &CareerBenchError) -> String {
    match error {
        CareerBenchError::Database(e) => match e {
            DatabaseError::ConnectionFailed(_) => "Database connection failed".to_string(),
            DatabaseError::QueryFailed(msg) => format!("Query failed: {}", msg),
            DatabaseError::MigrationFailed(_) => "Database migration failed".to_string(),
            DatabaseError::ConstraintViolation(msg) => msg.clone(),
            DatabaseError::NotFound(msg) => msg.clone(),
            DatabaseError::InvalidData(msg) => format!("Invalid data: {}", msg),
        },
        CareerBenchError::AiProvider(e) => {
            crate::ai::error_messages::get_short_error_message(e)
        }
        CareerBenchError::Validation(e) => match e {
            ValidationError::MissingField(field) => format!("{} is required", field),
            ValidationError::InvalidFormat(msg) => msg.clone(),
            ValidationError::OutOfRange(msg) => msg.clone(),
            ValidationError::BusinessRule(msg) => msg.clone(),
            ValidationError::General(msg) => msg.clone(),
        },
        CareerBenchError::Configuration(e) => match e {
            ConfigurationError::SettingNotFound(setting) => format!("{} not configured", setting),
            ConfigurationError::InvalidValue(msg) => msg.clone(),
            ConfigurationError::FileNotFound(_) => "Configuration file not found".to_string(),
            ConfigurationError::ParseError(msg) => msg.clone(),
            ConfigurationError::Other(msg) => msg.clone(),
        },
        CareerBenchError::FileSystem(e) => match e {
            FileSystemError::NotFound(path) => format!("File not found: {}", path),
            FileSystemError::PermissionDenied(path) => format!("Permission denied: {}", path),
            FileSystemError::DiskFull(_) => "Disk full".to_string(),
            FileSystemError::IoError(msg) => msg.clone(),
        },
        CareerBenchError::Application(msg) => msg.clone(),
    }
}

