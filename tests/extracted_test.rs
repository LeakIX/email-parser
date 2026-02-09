use email_extract::*;

#[test]
fn test_extract_emails() {
    let text = "Contact me at john@example.com or jane@company.org";
    let entities = ExtractedEntities::extract(text);

    assert_eq!(entities.emails.len(), 2);
    assert_eq!(entities.emails[0].address, "john@example.com");
    assert_eq!(entities.emails[1].address, "jane@company.org");
}

#[test]
fn test_extract_phone() {
    let text = "Call me at (555) 123-4567 or +1-555-987-6543";
    let entities = ExtractedEntities::extract(text);

    assert_eq!(entities.phone_numbers.len(), 2);
}

#[test]
fn test_extract_urls() {
    let text = "Visit https://example.com or \
                https://linkedin.com/in/johndoe";
    let entities = ExtractedEntities::extract(text);

    assert_eq!(entities.urls.len(), 2);
    assert_eq!(entities.urls[1].url_type, UrlType::SocialMedia);
}

#[test]
fn test_extract_amounts() {
    let text = "The price is $1,500.00 or €2000";
    let entities = ExtractedEntities::extract(text);

    assert_eq!(entities.amounts.len(), 2);
    assert_eq!(entities.amounts[0].value, 1500.0);
    assert_eq!(entities.amounts[0].currency, "USD");
}

#[test]
fn test_tracking_url_detection() {
    let text = "Visit https://click.example.com/track?id=123 \
                and https://example.com?utm_source=email \
                and https://example.com/about";
    let entities = ExtractedEntities::extract(text);

    assert_eq!(entities.urls.len(), 3);
    assert!(entities.urls[0].is_tracking);
    assert!(entities.urls[1].is_tracking);
    assert!(!entities.urls[2].is_tracking);
}
