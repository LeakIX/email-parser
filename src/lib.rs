// Enforce at crate level
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
#![allow(clippy::significant_drop_tightening)]

//! Intelligent Email Parser
//!
//! A modular, strongly-typed email parsing library that extracts structured
//! information from raw email data.
//!
//! # Features
//!
//! - Strong typing for all email components
//! - Automatic entity extraction (emails, phones, URLs, names)
//! - Sentiment analysis hints
//! - Language detection
//! - Thread analysis
//! - Spam indicators
//!
//! # Example
//!
//! ```rust
//! use email_parser::{Email, parse_email};
//!
//! let raw_email = b"From: sender@example.com\r\nSubject: Hello\r\n\r\nBody";
//! let email = parse_email(1, raw_email).unwrap();
//!
//! println!("From: {}", email.from);
//! println!("Extracted emails: {:?}", email.extracted.emails);
//! println!("Extracted phones: {:?}", email.extracted.phone_numbers);
//! ```

mod error;
mod extracted;
mod parser;
mod types;

pub use error::{ParseError, Result};
pub use extracted::*;
pub use parser::parse_email;
pub use types::*;
