# Changelog

All notable changes to this project will be documented in this file.

## 0.1.0

Initial release.

### Added

- Parse raw email bytes into strongly-typed `Email` structs
- Automatic entity extraction: emails, phone numbers, URLs, monetary
  amounts, social handles
- Thread analysis (reply depth, references, in-reply-to)
- Spam indicator detection and scoring
- Signature block separation
- HTML-to-text fallback for HTML-only emails
