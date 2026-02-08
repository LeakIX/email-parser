use email_parser::parse_email;

#[test]
fn test_parse_simple_email() {
    let raw = b"From: John Doe <john@example.com>\r\n\
                To: recipient@proton.me\r\n\
                Subject: Test Email\r\n\
                Date: Thu, 01 Jan 2025 12:00:00 +0000\r\n\
                Message-ID: <test123@example.com>\r\n\
                \r\n\
                Hello, this is a test email.";

    let email = parse_email(1, raw).unwrap();

    assert_eq!(email.uid, 1);
    assert_eq!(email.from.address, "john@example.com");
    assert_eq!(email.from.name.as_ref().unwrap().full, "John Doe");
    assert_eq!(email.subject.original, "Test Email");
    assert!(email.body.text.contains("test email"));
}

#[test]
fn test_parse_reply() {
    let raw = b"From: sender@example.com\r\n\
                To: recipient@example.com\r\n\
                Subject: Re: Re: Original Subject\r\n\
                Date: Thu, 01 Jan 2025 12:00:00 +0000\r\n\
                Message-ID: <reply@example.com>\r\n\
                In-Reply-To: <original@example.com>\r\n\
                \r\n\
                Reply content";

    let email = parse_email(1, raw).unwrap();

    assert_eq!(email.subject.reply_depth, 2);
    assert_eq!(email.subject.normalized, "Original Subject");
    assert!(email.thread.is_reply);
    assert!(email.thread.in_reply_to.is_some());
}

#[test]
fn test_extract_entities() {
    let raw = b"From: sender@example.com\r\n\
                To: recipient@example.com\r\n\
                Subject: Contact Info\r\n\
                Date: Thu, 01 Jan 2025 12:00:00 +0000\r\n\
                Message-ID: <test@example.com>\r\n\
                \r\n\
                Please contact me at john@company.com \
                or call (555) 123-4567.\n\
                Visit our website at https://company.com";

    let email = parse_email(1, raw).unwrap();

    assert!(!email.extracted.emails.is_empty());
    assert!(!email.extracted.phone_numbers.is_empty());
    assert!(!email.extracted.urls.is_empty());
}

#[test]
fn test_signature_separation() {
    let raw = b"From: sender@example.com\r\n\
                Subject: Test\r\n\
                Date: Thu, 01 Jan 2025 12:00:00 +0000\r\n\
                Message-ID: <sig@example.com>\r\n\
                \r\n\
                Hello there.\n\n--\nJohn Doe\nAcme Corp";

    let email = parse_email(1, raw).unwrap();

    assert_eq!(email.body.content_without_signature, "Hello there.");
    assert!(email.body.signature.is_some());
    assert!(email.body.signature.unwrap().contains("John Doe"));
}

#[test]
fn test_strip_html() {
    let raw = b"From: sender@example.com\r\n\
                Subject: Test\r\n\
                Date: Thu, 01 Jan 2025 12:00:00 +0000\r\n\
                Message-ID: <html@example.com>\r\n\
                Content-Type: text/html\r\n\
                \r\n\
                <html><body><h1>Hello</h1>\
                <p>World</p></body></html>";

    let email = parse_email(1, raw).unwrap();
    let text = email.body.text_from_html.unwrap();

    assert!(text.contains("Hello"));
    assert!(text.contains("World"));
    assert!(!text.contains("<"));
}
