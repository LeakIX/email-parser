//! Entity extraction from email content

use regex::Regex;
use serde::{Deserialize, Serialize};

/// All entities extracted from email content
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractedEntities {
    /// Email addresses found in body
    pub emails: Vec<ExtractedEmail>,

    /// Phone numbers found
    pub phone_numbers: Vec<PhoneNumber>,

    /// URLs found
    pub urls: Vec<ExtractedUrl>,

    /// Possible person names
    pub names: Vec<String>,

    /// Company names detected
    pub companies: Vec<String>,

    /// Dates mentioned
    pub dates: Vec<String>,

    /// Monetary amounts
    pub amounts: Vec<MonetaryAmount>,

    /// Physical addresses
    pub addresses: Vec<String>,

    /// Social media handles
    pub social_handles: Vec<SocialHandle>,
}

/// Extracted email address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEmail {
    pub address: String,
    pub context: String, // surrounding text
    pub position: usize, // character position in body
}

/// Phone number with type detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub raw: String,
    pub normalized: String,
    pub phone_type: PhoneType,
    pub country_code: Option<String>,
}

/// Type of phone number
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PhoneType {
    Mobile,
    Landline,
    TollFree,
    Unknown,
}

/// Extracted URL with analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedUrl {
    pub url: String,
    pub domain: String,
    pub is_tracking: bool,
    pub url_type: UrlType,
}

/// Type of URL
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UrlType {
    Website,
    SocialMedia,
    Unsubscribe,
    Tracking,
    Calendar,
    Document,
    Other,
}

/// Monetary amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetaryAmount {
    pub raw: String,
    pub value: f64,
    pub currency: String,
}

/// Social media handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialHandle {
    pub platform: SocialPlatform,
    pub handle: String,
}

/// Social media platform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SocialPlatform {
    Twitter,
    LinkedIn,
    Instagram,
    Facebook,
    GitHub,
    Other(String),
}

// Regex patterns
static EMAIL_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap()
});

static PHONE_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?:\+?1[-.\s]?)?(?:\(?\d{3}\)?[-.\s]?)?\d{3}[-.\s]?\d{4}").unwrap()
});

static URL_REGEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"https?://[^\s<>\[\]{}|\\^]+").unwrap());

static AMOUNT_REGEX: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"(?:[$€£¥])\s*[\d,]+(?:\.\d{2})?|[\d,]+(?:\.\d{2})?\s*(?:USD|EUR|GBP|CAD|AUD)")
        .unwrap()
});

static TWITTER_REGEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"@([a-zA-Z0-9_]{1,15})").unwrap());

static LINKEDIN_REGEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"linkedin\.com/in/([a-zA-Z0-9-]+)").unwrap());

/// Snap a byte index to the nearest valid UTF-8 char boundary (backwards)
const fn snap_to_char_boundary(s: &str, idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    let mut i = idx;
    while !s.is_char_boundary(i) && i > 0 {
        i -= 1;
    }
    i
}

impl ExtractedEntities {
    /// Extract all entities from text content
    pub fn extract(text: &str) -> Self {
        let mut entities = Self::default();

        // Extract emails
        for cap in EMAIL_REGEX.find_iter(text) {
            let start = snap_to_char_boundary(text, cap.start().saturating_sub(30));
            let end = snap_to_char_boundary(text, (cap.end() + 30).min(text.len()));
            let context = text[start..end].to_string();

            entities.emails.push(ExtractedEmail {
                address: cap.as_str().to_string(),
                context,
                position: cap.start(),
            });
        }

        // Extract phone numbers
        for cap in PHONE_REGEX.find_iter(text) {
            let raw = cap.as_str().to_string();
            let normalized = normalize_phone(&raw);
            let phone_type = detect_phone_type(&normalized);

            entities.phone_numbers.push(PhoneNumber {
                raw,
                normalized,
                phone_type,
                country_code: None,
            });
        }

        // Extract URLs
        for cap in URL_REGEX.find_iter(text) {
            let url = cap.as_str().to_string();
            let domain = extract_domain(&url);
            let is_tracking = is_tracking_url(&url);
            let url_type = detect_url_type(&url, &domain);

            entities.urls.push(ExtractedUrl {
                url,
                domain,
                is_tracking,
                url_type,
            });
        }

        // Extract monetary amounts
        for cap in AMOUNT_REGEX.find_iter(text) {
            if let Some(amount) = parse_amount(cap.as_str()) {
                entities.amounts.push(amount);
            }
        }

        // Extract social handles
        for cap in TWITTER_REGEX.captures_iter(text) {
            if let Some(handle) = cap.get(1) {
                entities.social_handles.push(SocialHandle {
                    platform: SocialPlatform::Twitter,
                    handle: handle.as_str().to_string(),
                });
            }
        }

        for cap in LINKEDIN_REGEX.captures_iter(text) {
            if let Some(handle) = cap.get(1) {
                entities.social_handles.push(SocialHandle {
                    platform: SocialPlatform::LinkedIn,
                    handle: handle.as_str().to_string(),
                });
            }
        }

        entities
    }

    /// Check if any entities were extracted
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.emails.is_empty()
            && self.phone_numbers.is_empty()
            && self.urls.is_empty()
            && self.amounts.is_empty()
    }

    /// Get count of all extracted entities
    #[must_use]
    pub const fn total_count(&self) -> usize {
        self.emails.len()
            + self.phone_numbers.len()
            + self.urls.len()
            + self.amounts.len()
            + self.social_handles.len()
    }
}

fn normalize_phone(phone: &str) -> String {
    phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect()
}

fn detect_phone_type(normalized: &str) -> PhoneType {
    let digits: String = normalized.chars().filter(char::is_ascii_digit).collect();

    if digits.starts_with("1800") || digits.starts_with("1888") || digits.starts_with("1877") {
        PhoneType::TollFree
    } else {
        PhoneType::Unknown
    }
}

fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

fn is_tracking_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("track")
        || lower.contains("click")
        || lower.contains("redirect")
        || lower.contains("utm_")
        || lower.contains("mc_eid")
        || lower.contains("trk")
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn detect_url_type(url: &str, domain: &str) -> UrlType {
    let lower = url.to_lowercase();
    let domain_lower = domain.to_lowercase();

    if lower.contains("unsubscribe") || lower.contains("optout") {
        UrlType::Unsubscribe
    } else if is_tracking_url(url) {
        UrlType::Tracking
    } else if domain_lower.contains("linkedin")
        || domain_lower.contains("twitter")
        || domain_lower.contains("facebook")
        || domain_lower.contains("instagram")
    {
        UrlType::SocialMedia
    } else if lower.contains("calendar") || lower.contains(".ics") {
        UrlType::Calendar
    } else if lower.ends_with(".pdf")
        || lower.ends_with(".doc")
        || lower.ends_with(".docx")
        || lower.ends_with(".xls")
    {
        UrlType::Document
    } else {
        UrlType::Website
    }
}

fn parse_amount(raw: &str) -> Option<MonetaryAmount> {
    let clean: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .collect();

    let clean = clean.replace(',', "");

    let value: f64 = clean.parse().ok()?;

    let currency = if raw.contains('$') || raw.contains("USD") {
        "USD"
    } else if raw.contains('€') || raw.contains("EUR") {
        "EUR"
    } else if raw.contains('£') || raw.contains("GBP") {
        "GBP"
    } else {
        "USD"
    };

    Some(MonetaryAmount {
        raw: raw.to_string(),
        value,
        currency: currency.to_string(),
    })
}
