use ai_coder_interface_rs::utils::*;

#[test]
fn test_human_readable_size() {
    assert_eq!(human_readable_size(0), "0.00 B");
    assert_eq!(human_readable_size(1023), "1023.00 B");
    assert_eq!(human_readable_size(1024), "1.00 KB");
    assert_eq!(human_readable_size(1500), "1.46 KB");
    assert_eq!(human_readable_size(1024 * 1024), "1.00 MB");
    assert_eq!(human_readable_size(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(human_readable_size(1024 * 1024 * 1024 * 1024), "1.00 TB");
}

#[test]
fn test_truncate_string() {
    assert_eq!(truncate_string("Hello", 10), "Hello");
    assert_eq!(truncate_string("Hello, world!", 10), "Hello,...");
    assert_eq!(truncate_string("This is a very long string", 15), "This is a v...");
    assert_eq!(truncate_string("", 10), "");
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(0), "0s");
    assert_eq!(format_duration(30), "30s");
    assert_eq!(format_duration(60), "1m 0s");
    assert_eq!(format_duration(90), "1m 30s");
    assert_eq!(format_duration(3600), "1h 0m 0s");
    assert_eq!(format_duration(3661), "1h 1m 1s");
}

#[test]
fn test_format_number() {
    assert_eq!(format_number(0), "0");
    assert_eq!(format_number(123), "123");
    assert_eq!(format_number(1234), "1,234");
    assert_eq!(format_number(1234567), "1,234,567");
    assert_eq!(format_number(1000000000), "1,000,000,000");
}

#[test]
fn test_format_float() {
    assert_eq!(format_float(0.0, 2), "0.00");
    assert_eq!(format_float(3.14159, 2), "3.14");
    assert_eq!(format_float(3.14159, 4), "3.1416");
    assert_eq!(format_float(1000.5, 1), "1000.5");
}

#[test]
fn test_format_money() {
    assert_eq!(format_money(0.0), "$0.0000");
    assert_eq!(format_money(1.5), "$1.5000");
    assert_eq!(format_money(1234.5678), "$1234.5678");
    assert_eq!(format_money(0.00001), "$0.0000");
}

#[test]
fn test_count_tokens() {
    assert_eq!(count_tokens(""), 0);
    assert_eq!(count_tokens("Hello"), 1);
    // Checking that our simple algorithm provides expected ranges
    let sample = "Hello world, this is a test.";
    let token_count = count_tokens(sample);
    assert!(token_count >= 5 && token_count <= 10, "Token count was: {}", token_count);
}