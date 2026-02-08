use email_parser::*;

// --- MessageId ---

#[test]
fn test_message_id_new() {
    let id = MessageId::new("test@example.com");
    assert_eq!(id.as_str(), "test@example.com");
}

#[test]
fn test_message_id_synthetic() {
    let id = MessageId::synthetic(42);
    assert_eq!(id.as_str(), "<synthetic-42@local>");
}

#[test]
fn test_message_id_display() {
    let id = MessageId::new("<abc@example.com>");
    assert_eq!(id.to_string(), "<abc@example.com>");
}

#[test]
fn test_message_id_equality() {
    let a = MessageId::new("same@id");
    let b = MessageId::new("same@id");
    assert_eq!(a, b);
}

// --- PersonName ---

#[test]
fn test_person_name_full() {
    let name = PersonName::parse("John Doe");
    assert_eq!(name.full, "John Doe");
    assert_eq!(name.first.as_deref(), Some("John"));
    assert_eq!(name.last.as_deref(), Some("Doe"));
}

#[test]
fn test_person_name_single() {
    let name = PersonName::parse("Madonna");
    assert_eq!(name.full, "Madonna");
    assert_eq!(name.first.as_deref(), Some("Madonna"));
    assert!(name.last.is_none());
}

#[test]
fn test_person_name_three_parts() {
    let name = PersonName::parse("John Michael Doe");
    assert_eq!(name.full, "John Michael Doe");
    assert_eq!(name.first.as_deref(), Some("John"));
    assert_eq!(name.last.as_deref(), Some("Doe"));
}

#[test]
fn test_person_name_empty() {
    let name = PersonName::parse("");
    assert_eq!(name.full, "");
    assert!(name.first.is_none());
    assert!(name.last.is_none());
}

#[test]
fn test_person_name_quoted() {
    let name = PersonName::parse("\"John Doe\"");
    assert_eq!(name.full, "John Doe");
    assert_eq!(name.first.as_deref(), Some("John"));
    assert_eq!(name.last.as_deref(), Some("Doe"));
}

#[test]
fn test_person_name_whitespace() {
    let name = PersonName::parse("  John   Doe  ");
    assert_eq!(name.full, "John   Doe");
    assert_eq!(name.first.as_deref(), Some("John"));
    assert_eq!(name.last.as_deref(), Some("Doe"));
}

#[test]
fn test_person_name_display() {
    let name = PersonName::parse("Alice Smith");
    assert_eq!(name.to_string(), "Alice Smith");
}

// --- EmailAddress ---

#[test]
fn test_email_address_parse_with_name() {
    let addr = EmailAddress::parse("John Doe <john@example.com>").unwrap();
    assert_eq!(addr.address, "john@example.com");
    assert_eq!(addr.domain, "example.com");
    assert_eq!(addr.local_part, "john");
    assert_eq!(addr.name.as_ref().unwrap().full, "John Doe");
}

#[test]
fn test_email_address_parse_plain() {
    let addr = EmailAddress::parse("alice@company.org").unwrap();
    assert_eq!(addr.address, "alice@company.org");
    assert_eq!(addr.domain, "company.org");
    assert_eq!(addr.local_part, "alice");
    assert!(addr.name.is_none());
}

#[test]
fn test_email_address_parse_angle_no_name() {
    let addr = EmailAddress::parse("<bob@test.io>").unwrap();
    assert_eq!(addr.address, "bob@test.io");
    assert!(addr.name.is_none());
}

#[test]
fn test_email_address_parse_quoted_name() {
    let addr = EmailAddress::parse("\"Jane Smith\" <jane@mail.com>").unwrap();
    assert_eq!(addr.name.as_ref().unwrap().full, "Jane Smith");
    assert_eq!(addr.address, "jane@mail.com");
}

#[test]
fn test_email_address_parse_invalid() {
    assert!(EmailAddress::parse("not-an-email").is_none());
}

#[test]
fn test_email_address_parse_empty() {
    assert!(EmailAddress::parse("").is_none());
}

#[test]
fn test_email_address_is_noreply() {
    let cases = [
        ("noreply@example.com", true),
        ("no-reply@example.com", true),
        ("donotreply@example.com", true),
        ("automated@example.com", true),
        ("mailer-daemon@example.com", true),
        ("john@example.com", false),
        ("support@example.com", false),
    ];
    for (addr, expected) in &cases {
        let parsed = EmailAddress::parse(addr).unwrap();
        assert_eq!(
            parsed.is_noreply(),
            *expected,
            "{addr} should be noreply={expected}"
        );
    }
}

#[test]
fn test_email_address_is_freemail() {
    let freemail = [
        "user@gmail.com",
        "user@yahoo.com",
        "user@outlook.com",
        "user@hotmail.com",
        "user@protonmail.com",
        "user@proton.me",
        "user@icloud.com",
        "user@aol.com",
    ];
    for addr in &freemail {
        let parsed = EmailAddress::parse(addr).unwrap();
        assert!(parsed.is_freemail(), "{addr} should be freemail");
    }

    let not_freemail = EmailAddress::parse("john@company.io").unwrap();
    assert!(!not_freemail.is_freemail());
}

#[test]
fn test_email_address_display_with_name() {
    let addr = EmailAddress::parse("Alice <alice@test.com>").unwrap();
    assert_eq!(addr.to_string(), "Alice <alice@test.com>");
}

#[test]
fn test_email_address_display_without_name() {
    let addr = EmailAddress::parse("bob@test.com").unwrap();
    assert_eq!(addr.to_string(), "bob@test.com");
}

// --- Subject ---

#[test]
fn test_subject_parse_plain() {
    let s = Subject::parse("Hello World");
    assert_eq!(s.original, "Hello World");
    assert_eq!(s.normalized, "Hello World");
    assert_eq!(s.reply_depth, 0);
    assert!(!s.is_forward);
}

#[test]
fn test_subject_parse_single_reply() {
    let s = Subject::parse("Re: Meeting Tomorrow");
    assert_eq!(s.normalized, "Meeting Tomorrow");
    assert_eq!(s.reply_depth, 1);
    assert!(!s.is_forward);
}

#[test]
fn test_subject_parse_nested_reply() {
    let s = Subject::parse("Re: Re: Re: Bug Report");
    assert_eq!(s.normalized, "Bug Report");
    assert_eq!(s.reply_depth, 3);
}

#[test]
fn test_subject_parse_reply_count_format() {
    let s = Subject::parse("Re[5]: Discussion Thread");
    assert_eq!(s.normalized, "Discussion Thread");
    assert_eq!(s.reply_depth, 5);
}

#[test]
fn test_subject_parse_forward() {
    let s = Subject::parse("Fwd: Important Document");
    assert!(s.is_forward);
    assert_eq!(s.reply_depth, 0);
}

#[test]
fn test_subject_parse_fw_short() {
    let s = Subject::parse("Fw: Short form forward");
    assert!(s.is_forward);
}

#[test]
fn test_subject_parse_reply_then_forward() {
    let s = Subject::parse("Re: Fwd: Nested Message");
    assert_eq!(s.reply_depth, 1);
    assert!(s.is_forward);
}

#[test]
fn test_subject_parse_case_insensitive() {
    let s = Subject::parse("RE: UPPERCASE REPLY");
    assert_eq!(s.reply_depth, 1);
    assert_eq!(s.normalized, "UPPERCASE REPLY");
}

#[test]
fn test_subject_display() {
    let s = Subject::parse("Re: Test");
    assert_eq!(s.to_string(), "Re: Test");
}

// --- Priority ---

#[test]
fn test_priority_from_header() {
    assert_eq!(Priority::from_header("1"), Priority::Highest);
    assert_eq!(Priority::from_header("2"), Priority::High);
    assert_eq!(Priority::from_header("3"), Priority::Normal);
    assert_eq!(Priority::from_header("4"), Priority::Low);
    assert_eq!(Priority::from_header("5"), Priority::Lowest);
    assert_eq!(Priority::from_header("invalid"), Priority::Normal);
    assert_eq!(Priority::from_header(" 1 "), Priority::Highest);
}

// --- Body ---

#[test]
fn test_body_is_empty() {
    let body = Body {
        text: String::new(),
        html: None,
        text_from_html: None,
        word_count: 0,
        char_count: 0,
        line_count: 0,
        language: None,
        has_attachments: false,
        signature: None,
        content_without_signature: String::new(),
    };
    assert!(body.is_empty());
}

#[test]
fn test_body_not_empty_with_text() {
    let body = Body {
        text: "Hello".to_string(),
        html: None,
        text_from_html: None,
        word_count: 1,
        char_count: 5,
        line_count: 1,
        language: None,
        has_attachments: false,
        signature: None,
        content_without_signature: "Hello".to_string(),
    };
    assert!(!body.is_empty());
}

#[test]
fn test_body_not_empty_with_html() {
    let body = Body {
        text: String::new(),
        html: Some("<p>Hi</p>".to_string()),
        text_from_html: None,
        word_count: 0,
        char_count: 0,
        line_count: 0,
        language: None,
        has_attachments: false,
        signature: None,
        content_without_signature: String::new(),
    };
    assert!(!body.is_empty());
}

#[test]
fn test_body_best_text_prefers_text() {
    let body = Body {
        text: "Plain text".to_string(),
        html: Some("<p>HTML</p>".to_string()),
        text_from_html: Some("From HTML".to_string()),
        word_count: 2,
        char_count: 10,
        line_count: 1,
        language: None,
        has_attachments: false,
        signature: None,
        content_without_signature: "Plain text".to_string(),
    };
    assert_eq!(body.best_text(), "Plain text");
}

#[test]
fn test_body_best_text_falls_back_to_html_text() {
    let body = Body {
        text: String::new(),
        html: Some("<p>HTML</p>".to_string()),
        text_from_html: Some("From HTML".to_string()),
        word_count: 0,
        char_count: 0,
        line_count: 0,
        language: None,
        has_attachments: false,
        signature: None,
        content_without_signature: String::new(),
    };
    assert_eq!(body.best_text(), "From HTML");
}

#[test]
fn test_body_best_text_empty() {
    let body = Body {
        text: String::new(),
        html: None,
        text_from_html: None,
        word_count: 0,
        char_count: 0,
        line_count: 0,
        language: None,
        has_attachments: false,
        signature: None,
        content_without_signature: String::new(),
    };
    assert_eq!(body.best_text(), "");
}

// --- Sentiment ---

#[test]
fn test_sentiment_default() {
    assert_eq!(Sentiment::default(), Sentiment::Neutral);
}
