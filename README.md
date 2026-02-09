# email-extract

Intelligent email parsing library for Rust, built on top of
[mailparse](https://crates.io/crates/mailparse). Parses raw email bytes into
strongly-typed structures with automatic entity extraction.

[API Documentation](https://leakix.github.io/email-extract)

## Features

- Strong typing for all email components (addresses, subjects, bodies, headers)
- Automatic entity extraction: emails, phone numbers, URLs, names, companies,
  monetary amounts, social handles
- Thread analysis (reply depth, references, in-reply-to)
- Spam indicator detection and scoring
- Signature block separation
- HTML-to-text fallback for HTML-only emails

## MSRV

The minimum supported Rust version is **1.90.0** (edition 2024).

## License

MIT
