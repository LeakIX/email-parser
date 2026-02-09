//! Intelligent email parsing with structured type extraction.
//!
//! Built on top of [mailparse](https://crates.io/crates/mailparse), this
//! library parses raw email bytes into strongly-typed structures with
//! automatic entity extraction.
//!
//! # Features
//!
//! - Strong typing for all email components
//! - Automatic entity extraction (emails, phones, URLs, names, companies,
//!   monetary amounts, social handles)
//! - Thread analysis (reply depth, references, in-reply-to)
//! - Spam indicator detection and scoring
//! - Signature block separation
//! - HTML-to-text fallback for HTML-only emails
//!
//! # Example
//!
//! ```rust
//! use email_extract::{Email, parse_email};
//!
//! let raw = b"From: alice@example.com\r\nSubject: Hello\r\n\r\nCall me at 555-1234";
//! let email = parse_email(1, raw).unwrap();
//!
//! assert_eq!(email.from.address, "alice@example.com");
//! assert_eq!(email.subject.original, "Hello");
//! assert!(!email.extracted.phone_numbers.is_empty());
//! ```

mod error;
mod extracted;
mod parser;
mod types;

pub use error::{ParseError, Result};
pub use extracted::*;
pub use parser::parse_email;
pub use types::*;
