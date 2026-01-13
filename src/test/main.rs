//! Main module tests - Version, constant-time comparison, and utility functions
//!
//! WHY: Ensure core utilities and security functions work correctly across different scenarios.

use crate::{Version, constant_time_eq, ct_eq_str_opt};

#[test]
fn test_version_display() {
    let version = Version::new();
    let display_str = version.to_string();

    assert!(
        display_str.contains("Profile"),
        "Version display should contain profile"
    );
}

#[test]
fn test_version_default() {
    let v1 = Version::new();
    let v2 = Version::default();

    assert_eq!(v1.to_string(), v2.to_string(), "Default should match new()");
}

#[test]
fn test_version_fields() {
    let version = Version::new();

    assert!(!version.version.is_empty(), "Version should have version");
    assert!(!version.profile.is_empty(), "Version should have profile");
    assert!(
        !version.commit_hash.is_empty(),
        "Version should have commit hash"
    );
}

#[test]
fn test_constant_time_eq_identical_strings() {
    let a = b"password";
    let b = b"password";

    assert!(
        constant_time_eq(a, b),
        "Identical passwords should be equal"
    );
}

#[test]
fn test_constant_time_eq_different_strings() {
    let a = b"password1";
    let b = b"password2";

    assert!(
        !constant_time_eq(a, b),
        "Different passwords should not be equal"
    );
}

#[test]
fn test_constant_time_eq_empty_strings() {
    let a = b"";
    let b = b"";

    assert!(constant_time_eq(a, b), "Empty strings should be equal");
}

#[test]
fn test_constant_time_eq_one_empty() {
    let a = b"";
    let b = b"nonempty";

    assert!(
        !constant_time_eq(a, b),
        "Empty and non-empty should not be equal"
    );
}

#[test]
fn test_constant_time_eq_length_diff() {
    let a = b"short";
    let b = b"much_longer_string";

    assert!(
        !constant_time_eq(a, b),
        "Different length strings should not be equal"
    );
}

#[test]
fn test_constant_time_eq_single_char_diff() {
    let a = b"abcdef";
    let b = b"abcdeg";

    assert!(
        !constant_time_eq(a, b),
        "Strings differing by one character should not be equal"
    );
}

#[test]
fn test_constant_time_eq_first_char_diff() {
    let a = b"xbcdef";
    let b = b"abcdef";

    assert!(
        !constant_time_eq(a, b),
        "Strings differing in first char should not be equal"
    );
}

#[test]
fn test_constant_time_eq_last_char_diff() {
    let a = b"abcdex";
    let b = b"abcdef";

    assert!(
        !constant_time_eq(a, b),
        "Strings differing in last char should not be equal"
    );
}

#[test]
fn test_constant_time_eq_timing_resistance() {
    // Test that we compare all bytes regardless of early differences
    let correct = b"secret_password_123";
    let wrong_early = b"x__________________";
    let wrong_late = b"secret_password_xx_";

    // All should return false
    assert!(!constant_time_eq(correct, wrong_early));
    assert!(!constant_time_eq(correct, wrong_late));

    // And be true for identical
    assert!(constant_time_eq(correct, correct));
}

#[test]
fn test_ct_eq_str_opt_both_some_equal() {
    let a = Some("password");
    let b = Some("password");

    assert!(ct_eq_str_opt(a, b), "Same values should be equal");
}

#[test]
fn test_ct_eq_str_opt_both_some_different() {
    let a = Some("password1");
    let b = Some("password2");

    assert!(!ct_eq_str_opt(a, b), "Different values should not be equal");
}

#[test]
fn test_ct_eq_str_opt_both_none() {
    let a: Option<&str> = None;
    let b: Option<&str> = None;

    assert!(ct_eq_str_opt(a, b), "Both None should be equal");
}

#[test]
fn test_ct_eq_str_opt_one_some_one_none() {
    let a = Some("password");
    let b: Option<&str> = None;

    assert!(!ct_eq_str_opt(a, b), "Some and None should not be equal");
}

#[test]
fn test_ct_eq_str_opt_none_some() {
    let a: Option<&str> = None;
    let b = Some("password");

    assert!(!ct_eq_str_opt(a, b), "None and Some should not be equal");
}

#[test]
fn test_ct_eq_str_opt_empty_string() {
    let a = Some("");
    let b = Some("");

    assert!(ct_eq_str_opt(a, b), "Empty strings should be equal");
}

#[test]
fn test_ct_eq_str_opt_one_empty() {
    let a = Some("");
    let b = Some("nonempty");

    assert!(
        !ct_eq_str_opt(a, b),
        "Empty and non-empty should not be equal"
    );
}

#[test]
fn test_constant_time_eq_unicode() {
    let a = "café".as_bytes();
    let b = "café".as_bytes();

    assert!(
        constant_time_eq(a, b),
        "Unicode strings should be comparable"
    );
}

#[test]
fn test_constant_time_eq_unicode_different() {
    let a = "café".as_bytes();
    let b = "cafe".as_bytes();

    assert!(
        !constant_time_eq(a, b),
        "Different unicode strings should not be equal"
    );
}

#[test]
fn test_constant_time_eq_large_inputs() {
    let large_a = vec![b'a'; 10000];
    let large_b = vec![b'a'; 10000];
    let large_c = vec![b'b'; 10000];

    assert!(
        constant_time_eq(&large_a, &large_b),
        "Large identical inputs should be equal"
    );
    assert!(
        !constant_time_eq(&large_a, &large_c),
        "Large different inputs should not be equal"
    );
}

#[test]
fn test_ct_eq_str_opt_long_strings() {
    let long_str = "a".repeat(1000);
    let a = Some(long_str.as_str());
    let b = Some(long_str.as_str());

    assert!(
        ct_eq_str_opt(a, b),
        "Long identical strings should be equal"
    );
}

#[test]
fn test_constant_time_eq_symmetric() {
    // Verify that comparison is symmetric
    let a = b"test";
    let b = b"different";

    assert_eq!(
        constant_time_eq(a, b),
        constant_time_eq(b, a),
        "Comparison should be symmetric"
    );
}

#[test]
fn test_constant_time_eq_reflexive() {
    let a = b"test";

    assert!(constant_time_eq(a, a), "Comparison should be reflexive");
}

#[test]
fn test_ct_eq_str_opt_reflexive() {
    let a = Some("test");

    assert!(ct_eq_str_opt(a, a), "Option comparison should be reflexive");
}
