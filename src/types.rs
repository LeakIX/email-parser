//! Core types for parsed emails

use crate::extracted::ExtractedEntities;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A fully parsed email with extracted entities and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    /// Unique message ID from headers
    pub message_id: MessageId,

    /// IMAP UID
    pub uid: u32,

    /// Sender address
    pub from: EmailAddress,

    /// Primary recipients
    pub to: Vec<EmailAddress>,

    /// CC recipients
    pub cc: Vec<EmailAddress>,

    /// BCC recipients (if available)
    pub bcc: Vec<EmailAddress>,

    /// Reply-To address (if different from From)
    pub reply_to: Option<EmailAddress>,

    /// Email subject
    pub subject: Subject,

    /// Email body content
    pub body: Body,

    /// Date sent/received
    pub date: DateTime<Utc>,

    /// Email headers
    pub headers: Headers,

    /// Thread information
    pub thread: ThreadInfo,

    /// Extracted entities from the email content
    pub extracted: ExtractedEntities,

    /// Email metadata and analysis
    pub metadata: EmailMetadata,
}

/// Message ID wrapper type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MessageId(pub String);

impl MessageId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a synthetic message ID if none provided
    #[must_use]
    pub fn synthetic(uid: u32) -> Self {
        Self(format!("<synthetic-{uid}@local>"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Email address with optional display name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailAddress {
    /// Display name (e.g., "John Doe")
    pub name: Option<PersonName>,

    /// Email address (e.g., "john@example.com")
    pub address: String,

    /// Domain extracted from address
    pub domain: String,

    /// Local part (before @)
    pub local_part: String,
}

impl EmailAddress {
    /// Parse an email address from a string
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();

        // Try to match "Name <email@domain.com>" format
        if let Some(start) = s.find('<')
            && let Some(end) = s.find('>')
        {
            let name_part = s[..start].trim().trim_matches('"');
            let address = s[start + 1..end].trim().to_string();

            if let Some((local, domain)) = address.split_once('@') {
                return Some(Self {
                    name: if name_part.is_empty() {
                        None
                    } else {
                        Some(PersonName::parse(name_part))
                    },
                    local_part: local.to_string(),
                    domain: domain.to_string(),
                    address,
                });
            }
        }

        // Plain email address
        if let Some((local, domain)) = s.split_once('@') {
            return Some(Self {
                name: None,
                local_part: local.to_string(),
                domain: domain.to_string(),
                address: s.to_string(),
            });
        }

        None
    }

    /// Check if this is likely a noreply/automated address
    #[must_use]
    pub fn is_noreply(&self) -> bool {
        let lower = self.local_part.to_lowercase();
        lower.contains("noreply")
            || lower.contains("no-reply")
            || lower.contains("donotreply")
            || lower.contains("automated")
            || lower.contains("mailer-daemon")
    }

    /// Check if this is from a known email service
    #[must_use]
    pub fn is_freemail(&self) -> bool {
        let domain = self.domain.to_lowercase();
        matches!(
            domain.as_str(),
            "gmail.com"
                | "yahoo.com"
                | "outlook.com"
                | "hotmail.com"
                | "protonmail.com"
                | "proton.me"
                | "icloud.com"
                | "aol.com"
        )
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} <{}>", name, self.address),
            None => write!(f, "{}", self.address),
        }
    }
}

/// Parsed person name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonName {
    /// Full name as provided
    pub full: String,

    /// First name (if parseable)
    pub first: Option<String>,

    /// Last name (if parseable)
    pub last: Option<String>,
}

impl PersonName {
    /// Parse a name string
    #[must_use]
    pub fn parse(s: &str) -> Self {
        let s = s.trim().trim_matches('"');
        let parts: Vec<&str> = s.split_whitespace().collect();

        match parts.len() {
            0 => Self {
                full: String::new(),
                first: None,
                last: None,
            },
            1 => Self {
                full: parts[0].to_string(),
                first: Some(parts[0].to_string()),
                last: None,
            },
            _ => Self {
                full: s.to_string(),
                first: Some(parts[0].to_string()),
                last: Some(parts[parts.len() - 1].to_string()),
            },
        }
    }
}

impl fmt::Display for PersonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full)
    }
}

/// Email subject with analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// Original subject line
    pub original: String,

    /// Subject without <Re:/Fwd>: prefixes
    pub normalized: String,

    /// Number of Re: prefixes (indicates thread depth)
    pub reply_depth: u32,

    /// Is this a forward?
    pub is_forward: bool,

    /// Detected language (ISO 639-1 code)
    pub language: Option<String>,
}

impl Subject {
    /// Parse a subject line
    #[must_use]
    pub fn parse(s: &str) -> Self {
        let mut normalized = s.to_string();
        let mut reply_depth = 0;
        let mut is_forward = false;

        // Count and remove Re: prefixes
        loop {
            let lower = normalized.to_lowercase();
            if lower.starts_with("re:") {
                normalized = normalized[3..].trim_start().to_string();
                reply_depth += 1;
            } else if lower.starts_with("re[") {
                // Handle Re[2]: format
                if let Some(end) = normalized.find("]:") {
                    if let Ok(count) = normalized[3..end].parse::<u32>() {
                        reply_depth += count;
                    }
                    normalized = normalized[end + 2..].trim_start().to_string();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Check for forward
        let lower = normalized.to_lowercase();
        if lower.starts_with("fwd:") || lower.starts_with("fw:") {
            is_forward = true;
            normalized = normalized
                .trim_start_matches(|c| {
                    c == 'F' || c == 'f' || c == 'w' || c == 'W' || c == 'd' || c == 'D' || c == ':'
                })
                .trim_start()
                .to_string();
        }

        Self {
            original: s.to_string(),
            normalized,
            reply_depth,
            is_forward,
            language: None, // Could add language detection
        }
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original)
    }
}

/// Email body content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    /// Plain text content
    pub text: String,

    /// HTML content (if available)
    pub html: Option<String>,

    /// Text extracted from HTML (if HTML-only email)
    pub text_from_html: Option<String>,

    /// Word count of text content
    pub word_count: usize,

    /// Character count
    pub char_count: usize,

    /// Line count
    pub line_count: usize,

    /// Detected language
    pub language: Option<String>,

    /// Has attachments indicator from content type
    pub has_attachments: bool,

    /// Signature block (if detected and separated)
    pub signature: Option<String>,

    /// Main content without signature
    pub content_without_signature: String,
}

impl Body {
    /// Check if body is empty or minimal
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty() && self.html.is_none()
    }

    /// Get the best available text content
    #[must_use]
    pub fn best_text(&self) -> &str {
        if !self.text.is_empty() {
            &self.text
        } else if let Some(ref html_text) = self.text_from_html {
            html_text
        } else {
            ""
        }
    }
}

/// Email headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Headers {
    /// All headers as key-value pairs
    pub all: Vec<(String, String)>,

    /// Content-Type
    pub content_type: Option<String>,

    /// X-Mailer or User-Agent
    pub mailer: Option<String>,

    /// X-Priority
    pub priority: Option<Priority>,

    /// List-Unsubscribe header (newsletters)
    pub list_unsubscribe: Option<String>,

    /// Authentication results
    pub authentication: AuthenticationResults,

    /// Custom headers (X-*)
    pub custom: Vec<(String, String)>,
}

/// Email priority level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Highest,
    High,
    Normal,
    Low,
    Lowest,
}

impl Priority {
    #[must_use]
    pub fn from_header(value: &str) -> Self {
        match value.trim() {
            "1" => Self::Highest,
            "2" => Self::High,
            "4" => Self::Low,
            "5" => Self::Lowest,
            _ => Self::Normal,
        }
    }
}

/// Email authentication results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthenticationResults {
    /// SPF result
    pub spf: Option<AuthResult>,

    /// DKIM result
    pub dkim: Option<AuthResult>,

    /// DMARC result
    pub dmarc: Option<AuthResult>,
}

/// Authentication result status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthResult {
    Pass,
    Fail,
    Neutral,
    None,
    Unknown(String),
}

/// Thread information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    /// In-Reply-To header (message ID of parent)
    pub in_reply_to: Option<MessageId>,

    /// References header (list of message IDs in thread)
    pub references: Vec<MessageId>,

    /// Is this a reply?
    pub is_reply: bool,

    /// Estimated position in thread
    pub thread_position: u32,
}

/// Email metadata and analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMetadata {
    /// Spam indicators score (0.0 = clean, 1.0 = spam)
    pub spam_score: f32,

    /// List of spam indicators found
    pub spam_indicators: Vec<SpamIndicator>,

    /// Urgency indicators
    pub urgency: Urgency,

    /// Email category hints
    pub category_hints: Vec<CategoryHint>,

    /// Is this likely automated/bulk mail?
    pub is_automated: bool,

    /// Is this from a mailing list?
    pub is_mailing_list: bool,

    /// Sentiment hints (positive, negative, neutral)
    pub sentiment: Sentiment,
}

/// Spam indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamIndicator {
    pub indicator: String,
    pub weight: f32,
}

/// Urgency level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Urgency {
    Critical,
    High,
    Normal,
    Low,
}

/// Category hint for email classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryHint {
    pub category: String,
    pub confidence: f32,
    pub reason: String,
}

/// Sentiment analysis result
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Sentiment {
    Positive,
    Negative,
    #[default]
    Neutral,
    Mixed,
}
