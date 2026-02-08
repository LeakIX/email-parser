//! Error types for email parsing

use thiserror::Error;

/// Errors that can occur during email parsing
#[derive(Error, Debug)]
pub enum ParseError {
    /// Failed to parse the email structure
    #[error("Failed to parse email structure: {0}")]
    Structure(String),

    /// Failed to decode email content
    #[error("Failed to decode content: {0}")]
    Decode(String),

    /// Missing required header
    #[error("Missing required header: {0}")]
    MissingHeader(String),

    /// Invalid header format
    #[error("Invalid header format for {header}: {details}")]
    InvalidHeader { header: String, details: String },

    /// Invalid date format
    #[error("Invalid date format: {0}")]
    InvalidDate(String),
}

/// Result type for email parsing operations
pub type Result<T> = std::result::Result<T, ParseError>;
