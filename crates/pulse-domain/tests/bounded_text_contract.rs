//! BoundedText safety tests — forbidden content rejection.

use pulse_domain::{BoundedText, DomainError};

#[test]
fn bounded_text_accepts_safe_provider_label() {
    assert!(BoundedText::new("Codex CLI").is_ok());
    assert!(BoundedText::new("Claude Code").is_ok());
    assert!(BoundedText::new("pulse-island").is_ok());
    assert!(BoundedText::new("task-42").is_ok());
}

#[test]
fn bounded_text_rejects_prompt() {
    assert!(matches!(
        BoundedText::new("user prompt text").unwrap_err(),
        DomainError::ForbiddenContent
    ));
}

#[test]
fn bounded_text_rejects_transcript() {
    assert!(matches!(
        BoundedText::new("model transcript content").unwrap_err(),
        DomainError::ForbiddenContent
    ));
}

#[test]
fn bounded_text_rejects_api_key() {
    assert!(matches!(
        BoundedText::new("api_key sk-abc").unwrap_err(),
        DomainError::ForbiddenContent
    ));
}

#[test]
fn bounded_text_rejects_secret() {
    assert!(matches!(
        BoundedText::new("my_secret_value").unwrap_err(),
        DomainError::ForbiddenContent
    ));
}

#[test]
fn bounded_text_rejects_token_equals() {
    assert!(matches!(
        BoundedText::new("token=abc123").unwrap_err(),
        DomainError::ForbiddenContent
    ));
}

#[test]
fn bounded_text_rejects_password() {
    assert!(matches!(
        BoundedText::new("password hunter2").unwrap_err(),
        DomainError::ForbiddenContent
    ));
}

#[test]
fn bounded_text_rejects_credential() {
    assert!(matches!(
        BoundedText::new("credential xyz").unwrap_err(),
        DomainError::ForbiddenContent
    ));
}

#[test]
fn bounded_text_rejects_bearer() {
    assert!(matches!(
        BoundedText::new("bearer token").unwrap_err(),
        DomainError::ForbiddenContent
    ));
}

#[test]
fn forbidden_check_is_case_insensitive() {
    assert!(matches!(
        BoundedText::new("PROMPT text").unwrap_err(),
        DomainError::ForbiddenContent
    ));
    assert!(matches!(
        BoundedText::new("BEARER xyz").unwrap_err(),
        DomainError::ForbiddenContent
    ));
}

#[test]
fn bounded_text_rejects_over_max_length() {
    let long = "a".repeat(65);
    assert!(matches!(
        BoundedText::new(&long).unwrap_err(),
        DomainError::TooLong
    ));
}

#[test]
fn bounded_text_round_trips_as_str() {
    let text = BoundedText::new("hello-world").unwrap();
    assert_eq!(text.as_str(), "hello-world");
}

#[test]
fn bounded_text_exactly_max_length_ok() {
    let exact = "a".repeat(64);
    assert!(BoundedText::new(&exact).is_ok());
}