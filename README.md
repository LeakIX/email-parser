# email-parser

Intelligent email parsing library for Rust, built on top of
[mailparse](https://crates.io/crates/mailparse). Parses raw email bytes into
strongly-typed structures with automatic entity extraction.

[API Documentation](https://leakix.github.io/email-parser)

## Features

- Strong typing for all email components (addresses, subjects, bodies, headers)
- Automatic entity extraction: emails, phone numbers, URLs, names, companies,
  monetary amounts, social handles
- Thread analysis (reply depth, references, in-reply-to)
- Spam indicator detection and scoring
- Signature block separation
- HTML-to-text fallback for HTML-only emails

## Usage

```rust
use email_parser::{Email, parse_email};

let raw = b"From: alice@example.com\r\nSubject: Hello\r\n\r\nCall me at 555-1234";
let email = parse_email(1, raw).unwrap();

assert_eq!(email.from.address, "alice@example.com");
assert_eq!(email.subject.original, "Hello");
assert!(!email.extracted.phone_numbers.is_empty());
```

## MSRV

The minimum supported Rust version is **1.90.0** (edition 2024).

## License

MIT
