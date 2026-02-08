//! Main email parser implementation

use crate::error::{ParseError, Result};
use crate::extracted::ExtractedEntities;
use crate::types::{
    AuthResult, AuthenticationResults, Body, CategoryHint, Email, EmailAddress, EmailMetadata,
    Headers, MessageId, Priority, Sentiment, SpamIndicator, Subject, ThreadInfo, Urgency,
};
use chrono::{DateTime, Utc};
use tracing::debug;

/// Parse raw email bytes into a structured Email
pub fn parse_email(uid: u32, raw: &[u8]) -> Result<Email> {
    let parsed = mailparse::parse_mail(raw).map_err(|e| ParseError::Structure(e.to_string()))?;

    let headers = parse_headers(&parsed.headers)?;
    let message_id = extract_message_id(&parsed.headers, uid);
    let from = extract_from(&parsed.headers)?;
    let to = extract_addresses(&parsed.headers, "to");
    let cc = extract_addresses(&parsed.headers, "cc");
    let bcc = extract_addresses(&parsed.headers, "bcc");
    let reply_to = extract_reply_to(&parsed.headers);
    let subject = extract_subject(&parsed.headers);
    let date = extract_date(&parsed.headers);
    let thread = extract_thread_info(&parsed.headers, &subject);
    let body = extract_body(&parsed);

    // Extract entities from body
    let extracted = ExtractedEntities::extract(body.best_text());

    // Analyze email metadata
    let metadata = analyze_metadata(&from, &headers, &subject, &body, &extracted);

    debug!("Parsed email: {} from {}", subject.original, from.address);

    Ok(Email {
        message_id,
        uid,
        from,
        to,
        cc,
        bcc,
        reply_to,
        subject,
        body,
        date,
        headers,
        thread,
        extracted,
        metadata,
    })
}

#[allow(clippy::unnecessary_wraps)]
fn parse_headers(headers: &[mailparse::MailHeader]) -> Result<Headers> {
    let all: Vec<(String, String)> = headers
        .iter()
        .map(|h| (h.get_key().to_lowercase(), h.get_value()))
        .collect();

    let content_type = headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "content-type")
        .map(mailparse::MailHeader::get_value);

    let mailer = headers
        .iter()
        .find(|h| {
            let key = h.get_key().to_lowercase();
            key == "x-mailer" || key == "user-agent"
        })
        .map(mailparse::MailHeader::get_value);

    let priority = headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "x-priority")
        .map(|h| Priority::from_header(&h.get_value()));

    let list_unsubscribe = headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "list-unsubscribe")
        .map(mailparse::MailHeader::get_value);

    let authentication = parse_authentication_results(headers);

    let custom: Vec<(String, String)> = headers
        .iter()
        .filter(|h| h.get_key().to_lowercase().starts_with("x-"))
        .map(|h| (h.get_key(), h.get_value()))
        .collect();

    Ok(Headers {
        all,
        content_type,
        mailer,
        priority,
        list_unsubscribe,
        authentication,
        custom,
    })
}

fn parse_authentication_results(headers: &[mailparse::MailHeader]) -> AuthenticationResults {
    let mut results = AuthenticationResults::default();

    for header in headers {
        if header.get_key().to_lowercase() == "authentication-results" {
            let value = header.get_value().to_lowercase();

            if value.contains("spf=pass") {
                results.spf = Some(AuthResult::Pass);
            } else if value.contains("spf=fail") {
                results.spf = Some(AuthResult::Fail);
            }

            if value.contains("dkim=pass") {
                results.dkim = Some(AuthResult::Pass);
            } else if value.contains("dkim=fail") {
                results.dkim = Some(AuthResult::Fail);
            }

            if value.contains("dmarc=pass") {
                results.dmarc = Some(AuthResult::Pass);
            } else if value.contains("dmarc=fail") {
                results.dmarc = Some(AuthResult::Fail);
            }
        }
    }

    results
}

fn extract_message_id(headers: &[mailparse::MailHeader], uid: u32) -> MessageId {
    headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "message-id")
        .map_or_else(
            || MessageId::synthetic(uid),
            |h| MessageId::new(h.get_value()),
        )
}

fn extract_from(headers: &[mailparse::MailHeader]) -> Result<EmailAddress> {
    let from_header = headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "from")
        .map(mailparse::MailHeader::get_value)
        .ok_or_else(|| ParseError::MissingHeader("From".into()))?;

    EmailAddress::parse(&from_header).ok_or_else(|| ParseError::InvalidHeader {
        header: "From".into(),
        details: format!("Could not parse: {from_header}"),
    })
}

fn extract_addresses(headers: &[mailparse::MailHeader], header_name: &str) -> Vec<EmailAddress> {
    headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == header_name)
        .map(|h| {
            h.get_value()
                .split(',')
                .filter_map(|addr| EmailAddress::parse(addr.trim()))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_reply_to(headers: &[mailparse::MailHeader]) -> Option<EmailAddress> {
    headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "reply-to")
        .and_then(|h| EmailAddress::parse(&h.get_value()))
}

fn extract_subject(headers: &[mailparse::MailHeader]) -> Subject {
    let subject_text = headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "subject")
        .map_or_else(
            || "(no subject)".to_string(),
            mailparse::MailHeader::get_value,
        );

    Subject::parse(&subject_text)
}

fn extract_date(headers: &[mailparse::MailHeader]) -> DateTime<Utc> {
    headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "date")
        .and_then(|h| DateTime::parse_from_rfc2822(&h.get_value()).ok())
        .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc))
}

fn extract_thread_info(headers: &[mailparse::MailHeader], subject: &Subject) -> ThreadInfo {
    let in_reply_to = headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "in-reply-to")
        .map(|h| MessageId::new(h.get_value()));

    let references: Vec<MessageId> = headers
        .iter()
        .find(|h| h.get_key().to_lowercase() == "references")
        .map(|h| {
            h.get_value()
                .split_whitespace()
                .map(|s| MessageId::new(s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let is_reply = in_reply_to.is_some() || subject.reply_depth > 0;
    #[allow(clippy::cast_possible_truncation)]
    let thread_position = if is_reply {
        references.len() as u32 + 1
    } else {
        0
    };

    ThreadInfo {
        in_reply_to,
        references,
        is_reply,
        thread_position,
    }
}

fn extract_body(parsed: &mailparse::ParsedMail) -> Body {
    let (text, html) = extract_body_parts(parsed);

    // Extract text from HTML if no plain text
    let text_from_html = if text.is_empty() {
        html.as_ref().map(|h| strip_html(h))
    } else {
        None
    };

    let best_text = if !text.is_empty() {
        &text
    } else if let Some(ref html_text) = text_from_html {
        html_text
    } else {
        ""
    };

    // Separate signature from content
    let (content_without_signature, signature) = separate_signature(best_text);

    Body {
        word_count: best_text.split_whitespace().count(),
        char_count: best_text.len(),
        line_count: best_text.lines().count(),
        text,
        html,
        text_from_html,
        language: None,         // Could add language detection
        has_attachments: false, // Would need to check multipart structure
        signature,
        content_without_signature,
    }
}

fn extract_body_parts(parsed: &mailparse::ParsedMail) -> (String, Option<String>) {
    let mut text = String::new();
    let mut html: Option<String> = None;

    if parsed.subparts.is_empty() {
        let content_type = parsed.ctype.mimetype.to_lowercase();
        if let Ok(body) = parsed.get_body() {
            if content_type.contains("text/html") {
                html = Some(body);
            } else {
                text = body;
            }
        }
    } else {
        extract_body_recursive(parsed, &mut text, &mut html);
    }

    (text, html)
}

fn extract_body_recursive(
    parsed: &mailparse::ParsedMail,
    text: &mut String,
    html: &mut Option<String>,
) {
    for part in &parsed.subparts {
        let content_type = part.ctype.mimetype.to_lowercase();

        if part.subparts.is_empty() {
            if let Ok(body) = part.get_body() {
                if content_type.contains("text/plain") && text.is_empty() {
                    *text = body;
                } else if content_type.contains("text/html") && html.is_none() {
                    *html = Some(body);
                }
            }
        } else {
            extract_body_recursive(part, text, html);
        }
    }
}

fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_start_idx: usize = 0;

    let lower_chars: Vec<char> = html.to_lowercase().chars().collect();
    let chars: Vec<char> = html.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        if !in_tag && chars[i] == '<' {
            tag_start_idx = i;
            // Check for script/style start via char slice
            let remaining: String = lower_chars[i..].iter().collect();
            if remaining.starts_with("<script") {
                in_script = true;
            } else if remaining.starts_with("<style") {
                in_style = true;
            } else if remaining.starts_with("</script") {
                in_script = false;
            } else if remaining.starts_with("</style") {
                in_style = false;
            }
            in_tag = true;
        } else if in_tag && chars[i] == '>' {
            in_tag = false;
            // Add newline after block elements
            let tag_content: String = lower_chars[tag_start_idx + 1..i].iter().collect();
            if tag_content.starts_with("br")
                || tag_content.starts_with("/p")
                || tag_content.starts_with("/div")
                || tag_content.starts_with("/li")
                || tag_content.starts_with("/h")
            {
                result.push('\n');
            }
        } else if !in_tag && !in_script && !in_style {
            result.push(chars[i]);
        }
        i += 1;
    }

    // Decode HTML entities
    result = result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Clean up whitespace
    result
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn separate_signature(text: &str) -> (String, Option<String>) {
    // Common signature delimiters
    let delimiters = [
        "--\n",
        "-- \n",
        "---\n",
        "Best regards",
        "Kind regards",
        "Regards,",
    ];

    for delimiter in delimiters {
        if let Some(pos) = text.find(delimiter) {
            let content = text[..pos].trim().to_string();
            let signature = text[pos..].trim().to_string();
            if !signature.is_empty() {
                return (content, Some(signature));
            }
        }
    }

    (text.to_string(), None)
}

fn analyze_metadata(
    from: &EmailAddress,
    headers: &Headers,
    subject: &Subject,
    body: &Body,
    extracted: &ExtractedEntities,
) -> EmailMetadata {
    let mut spam_indicators = Vec::new();
    let mut spam_score: f32 = 0.0;

    // Check spam indicators
    if from.is_noreply() {
        spam_indicators.push(SpamIndicator {
            indicator: "noreply_sender".into(),
            weight: 0.1,
        });
        spam_score += 0.1;
    }

    // Check for tracking URLs
    let tracking_count = extracted.urls.iter().filter(|u| u.is_tracking).count();
    if tracking_count > 3 {
        spam_indicators.push(SpamIndicator {
            indicator: "excessive_tracking".into(),
            weight: 0.2,
        });
        spam_score += 0.2;
    }

    // Check subject for spam patterns
    let subject_lower = subject.original.to_lowercase();
    if subject_lower.contains("urgent")
        || subject_lower.contains("act now")
        || subject_lower.contains("limited time")
    {
        spam_indicators.push(SpamIndicator {
            indicator: "urgency_language".into(),
            weight: 0.15,
        });
        spam_score += 0.15;
    }

    // Determine urgency
    let urgency = if subject_lower.contains("urgent")
        || subject_lower.contains("asap")
        || subject_lower.contains("emergency")
        || headers.priority == Some(Priority::High)
        || headers.priority == Some(Priority::Highest)
    {
        Urgency::High
    } else {
        Urgency::Normal
    };

    // Category hints
    let mut category_hints = Vec::new();

    if headers.list_unsubscribe.is_some() {
        category_hints.push(CategoryHint {
            category: "newsletter".into(),
            confidence: 0.9,
            reason: "Has List-Unsubscribe header".into(),
        });
    }

    if from.is_noreply() {
        category_hints.push(CategoryHint {
            category: "automated".into(),
            confidence: 0.8,
            reason: "From noreply address".into(),
        });
    }

    if !extracted.phone_numbers.is_empty() && !extracted.companies.is_empty() {
        category_hints.push(CategoryHint {
            category: "lead".into(),
            confidence: 0.6,
            reason: "Contains contact information".into(),
        });
    }

    let is_automated = from.is_noreply() || headers.mailer.is_some();
    let is_mailing_list = headers.list_unsubscribe.is_some();

    // Simple sentiment detection
    let text_lower = body.best_text().to_lowercase();
    let sentiment = if text_lower.contains("thank")
        || text_lower.contains("appreciate")
        || text_lower.contains("great")
        || text_lower.contains("excellent")
    {
        Sentiment::Positive
    } else if text_lower.contains("complaint")
        || text_lower.contains("frustrated")
        || text_lower.contains("disappointed")
        || text_lower.contains("problem")
    {
        Sentiment::Negative
    } else {
        Sentiment::Neutral
    };

    EmailMetadata {
        spam_score: spam_score.min(1.0),
        spam_indicators,
        urgency,
        category_hints,
        is_automated,
        is_mailing_list,
        sentiment,
    }
}
