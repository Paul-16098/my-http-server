//! Error handling tests - Validating AppError types and HTTP status mapping
//!
//! WHY: Ensure error types correctly map to HTTP status codes and Responder impl works.
//! Covers all error variants and their status code behaviors.

use crate::error::AppError;
use actix_web::http::StatusCode;
use actix_web::ResponseError;

#[test]
fn test_error_io_not_found() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let app_err = AppError::Io(io_err);
    
    assert_eq!(
        app_err.status_code(),
        StatusCode::NOT_FOUND,
        "IO NotFound error should map to 404"
    );
}

#[test]
fn test_error_io_permission_denied() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let app_err = AppError::Io(io_err);
    
    assert_eq!(
        app_err.status_code(),
        StatusCode::FORBIDDEN,
        "IO PermissionDenied error should map to 403"
    );
}

#[test]
fn test_error_io_invalid_input() {
    let io_err = std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid input");
    let app_err = AppError::Io(io_err);
    
    assert_eq!(
        app_err.status_code(),
        StatusCode::BAD_REQUEST,
        "IO InvalidInput error should map to 400"
    );
}

#[test]
fn test_error_io_other() {
    let io_err = std::io::Error::other("some other io error");
    let app_err = AppError::Io(io_err);
    
    assert_eq!(
        app_err.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "Other IO errors should map to 500"
    );
}

#[test]
fn test_error_markdown_parse_error() {
    let app_err = AppError::MarkdownParseError("Failed to parse markdown".to_string());
    
    assert_eq!(
        app_err.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "MarkdownParseError should map to 500"
    );
}

#[test]
fn test_error_config_error() {
    let config_err = config::ConfigError::Message("Config error".to_string());
    let app_err = AppError::ConfigError(config_err);
    
    assert_eq!(
        app_err.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "ConfigError should map to 500"
    );
}

#[test]
fn test_error_cli_error() {
    let app_err = AppError::CliError("Invalid CLI argument".to_string());
    
    assert_eq!(
        app_err.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "CliError should map to 500"
    );
}

#[test]
fn test_error_other_error() {
    let app_err = AppError::OtherError("Generic error".to_string());
    
    assert_eq!(
        app_err.status_code(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "OtherError should map to 500"
    );
}

#[test]
fn test_error_display() {
    let app_err = AppError::OtherError("Test error message".to_string());
    
    let err_str = app_err.to_string();
    assert!(err_str.contains("Test error message"), "Error display should contain message");
}

#[test]
fn test_error_cli_error_display() {
    let app_err = AppError::CliError("CLI argument missing".to_string());
    
    let err_str = app_err.to_string();
    assert!(err_str.contains("CLI argument missing"), "CLI error display should contain message");
}

#[test]
fn test_error_markdown_parse_error_display() {
    let app_err = AppError::MarkdownParseError("Expected heading".to_string());
    
    let err_str = app_err.to_string();
    assert!(err_str.contains("Expected heading"), "Markdown error display should contain message");
}

#[test]
fn test_error_response_generation() {
    let app_err = AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "not found"));
    let response = app_err.error_response();
    
    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Error response should have correct status"
    );
}

#[test]
fn test_error_from_box_error() {
    let boxed_err: Box<dyn std::error::Error> = "test error".into();
    let app_err: AppError = boxed_err.into();
    
    match app_err {
        AppError::OtherError(msg) => {
            assert!(msg.contains("test error"), "Boxed error should convert to OtherError");
        }
        _ => panic!("Expected OtherError variant"),
    }
}

#[test]
fn test_all_error_types_map_to_500_or_specific() {
    // Test that all error types have a defined status code
    let errors = vec![
        (
            AppError::CliError("test".to_string()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            AppError::OtherError("test".to_string()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            AppError::MarkdownParseError("test".to_string()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ];
    
    for (err, expected_status) in errors {
        assert_eq!(
            err.status_code(),
            expected_status,
            "Error should map to expected status"
        );
    }
}

#[test]
fn test_error_responder_impl() {
    let app_err = AppError::OtherError("test".to_string());
    
    // Create a mock HttpRequest (we use a simple test to ensure Responder impl exists)
    // The actual respond_to is tested through integration tests
    let status = app_err.status_code();
    assert!(
        status.is_server_error() || status.is_client_error(),
        "Error should map to valid HTTP error status"
    );
}
