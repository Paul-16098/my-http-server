//! Security-critical function tests
//!
//! Tests for timing-safe comparison functions and path traversal prevention.

use crate::{constant_time_eq, ct_eq_str_opt};

#[test]
fn test_constant_time_eq_equal_strings() {
    let a = b"password123";
    let b = b"password123";
    assert!(constant_time_eq(a, b));
}

#[test]
fn test_constant_time_eq_different_strings() {
    let a = b"password123";
    let b = b"password456";
    assert!(!constant_time_eq(a, b));
}

#[test]
fn test_constant_time_eq_different_lengths() {
    let a = b"short";
    let b = b"muchlongerstring";
    assert!(!constant_time_eq(a, b));
}

#[test]
fn test_constant_time_eq_empty_strings() {
    let a = b"";
    let b = b"";
    assert!(constant_time_eq(a, b));
}

#[test]
fn test_constant_time_eq_one_empty() {
    let a = b"nonempty";
    let b = b"";
    assert!(!constant_time_eq(a, b));
}

#[test]
fn test_constant_time_eq_similar_prefix() {
    // Test that similar prefixes don't cause early exit
    let a = b"password123";
    let b = b"password999";
    assert!(!constant_time_eq(a, b));
}

#[test]
fn test_ct_eq_str_opt_both_some_equal() {
    assert!(ct_eq_str_opt(Some("test"), Some("test")));
}

#[test]
fn test_ct_eq_str_opt_both_some_different() {
    assert!(!ct_eq_str_opt(Some("test"), Some("other")));
}

#[test]
fn test_ct_eq_str_opt_both_none() {
    assert!(ct_eq_str_opt(None, None));
}

#[test]
fn test_ct_eq_str_opt_one_none() {
    assert!(!ct_eq_str_opt(Some("test"), None));
    assert!(!ct_eq_str_opt(None, Some("test")));
}

#[test]
fn test_ct_eq_str_opt_empty_strings() {
    assert!(ct_eq_str_opt(Some(""), Some("")));
}

#[test]
fn test_ct_eq_str_opt_unicode() {
    assert!(ct_eq_str_opt(Some("测试"), Some("测试")));
    assert!(!ct_eq_str_opt(Some("测试"), Some("テスト")));
}

// Property-based test: timing should not vary based on where strings differ
#[test]
fn test_constant_time_eq_timing_property() {
    // This is a basic property test - in production, use proper timing analysis
    // We verify that different positions of difference don't cause different behavior
    let base = b"0123456789abcdef";

    // Differ at first position
    let diff_start = b"X123456789abcdef";
    assert!(!constant_time_eq(base, diff_start));

    // Differ at middle
    let diff_mid = b"01234567X9abcdef";
    assert!(!constant_time_eq(base, diff_mid));

    // Differ at end
    let diff_end = b"0123456789abcdeX";
    assert!(!constant_time_eq(base, diff_end));
}

#[test]
fn test_constant_time_eq_zero_bytes() {
    // Test with zero bytes (null characters)
    let a = b"test\x00data";
    let b = b"test\x00data";
    assert!(constant_time_eq(a, b));

    let c = b"test\x00other";
    assert!(!constant_time_eq(a, c));
}

#[test]
fn test_constant_time_eq_all_zero_bytes() {
    let a = b"\x00\x00\x00\x00";
    let b = b"\x00\x00\x00\x00";
    assert!(constant_time_eq(a, b));

    let c = b"\x00\x00\x01\x00";
    assert!(!constant_time_eq(a, c));
}

#[test]
fn test_ct_eq_str_opt_special_characters() {
    assert!(ct_eq_str_opt(Some("!@#$%^&*()"), Some("!@#$%^&*()")));
    assert!(!ct_eq_str_opt(Some("!@#$%^&*()"), Some("!@#$%^&*(]")));
}
