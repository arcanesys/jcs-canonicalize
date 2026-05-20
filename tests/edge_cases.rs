//! Targeted edge cases that distinguish RFC 8785-correct implementations
//! from naive ones. These complement the cyberphone reference corpus by
//! exercising specific failure modes that would not show up against simpler
//! inputs.

use jcs_canonicalize::canonicalize;

/// Non-BMP key sort: RFC 8785 §3.2.3 mandates ordering by UTF-16 code units,
/// not by UTF-8 byte sequence. The two orderings disagree as soon as a key
/// contains a non-BMP code point (encoded as a surrogate pair starting with
/// 0xD800-0xDBFF in UTF-16, but with 0xF0+ leading byte in UTF-8).
///
/// Concrete pair:
///   "\u{20BB7}" (𠮷, CJK ideograph) - UTF-16: D842 DFB7 - UTF-8: F0 A0 AE B7
///   "\u{FFA5}"  (ﾥ, halfwidth katakana) - UTF-16: FFA5 - UTF-8: EF BE A5
///
/// UTF-8 byte order:        EF BE A5 < F0 A0 AE B7  ->  "\u{FFA5}" first
/// UTF-16 code unit order:  D842     < FFA5         ->  "\u{20BB7}" first
///
/// RFC-correct output: the non-BMP key sorts first.
#[test]
fn non_bmp_keys_sort_by_utf16_code_units_not_utf8_bytes() {
    let input = "{\"\u{FFA5}\":1,\"\u{20BB7}\":2}";
    let produced = canonicalize(input).expect("canonicalize");
    assert_eq!(
        produced, "{\"\u{20BB7}\":2,\"\u{FFA5}\":1}",
        "non-BMP key must sort before \\uFFA5 by UTF-16 code units"
    );
}

/// RFC 8785 §3.2.2.3 (via ECMAScript Number.prototype.toString) switches
/// from positional notation to exponent notation at specific magnitudes.
/// These boundary cases are where naive number formatters diverge.
#[test]
fn ecmascript_number_to_string_boundaries() {
    let cases: &[(&str, &str)] = &[
        // 1e21 is the first magnitude where ES uses exponent form.
        ("1e21", "1e+21"),
        // 1e20 still uses positional form: 100000000000000000000.
        ("1e20", "100000000000000000000"),
        // 1e-6 is positional, 1e-7 is exponent (just below the cutoff).
        ("1e-6", "0.000001"),
        ("1e-7", "1e-7"),
        // Negative exponent at boundary.
        ("0.1", "0.1"),
        // Trailing zeros are dropped.
        ("1.50", "1.5"),
        ("10.0", "10"),
    ];
    let mut failures = Vec::new();
    for (input, expected) in cases {
        let wrapped = format!(r#"{{"n":{input}}}"#);
        let want = format!(r#"{{"n":{expected}}}"#);
        let got = canonicalize(&wrapped).expect("canonicalize");
        if got != want {
            failures.push(format!("input={input} expected={expected} got={got}"));
        }
    }
    assert!(failures.is_empty(), "number boundary failures:\n{}", failures.join("\n"));
}

/// Negative zero is serialized as `0` (RFC 8785 §3.2.2.3, mirroring ES
/// Number.prototype.toString which treats -0 and +0 identically).
#[test]
fn negative_zero_is_serialized_as_zero() {
    let produced = canonicalize(r#"{"n":-0}"#).expect("canonicalize");
    assert_eq!(produced, r#"{"n":0}"#);
}

/// NaN and +/-Infinity are not valid JSON values; canonicalization MUST
/// reject any path that would produce them. The library cannot receive
/// these from a JSON string input (the parser rejects them first), but the
/// rejection chain is part of the conformance contract.
#[test]
fn non_finite_numbers_are_rejected_at_parse() {
    assert!(canonicalize(r#"{"n":NaN}"#).is_err());
    assert!(canonicalize(r#"{"n":Infinity}"#).is_err());
    assert!(canonicalize(r#"{"n":-Infinity}"#).is_err());
}

/// RFC 8785 inherits RFC 8259 string escape rules: only `"`, `\`, and the
/// C0 controls U+0000-U+001F must be escaped on output; everything else
/// (including `/` and U+007F DEL) is emitted as the literal UTF-8 code
/// point. The `\uXXXX` escape form MUST use lowercase hex.
#[test]
fn ascii_del_is_not_escaped_but_c0_controls_are() {
    // U+007F (DEL) is NOT a C0 control: literal in -> literal out.
    let produced = canonicalize("{\"k\":\"\u{007F}\"}").expect("canonicalize");
    assert_eq!(produced, "{\"k\":\"\u{007F}\"}");

    // RFC 8259 forbids literal C0 controls in JSON source; they must be
    // written as \uXXXX. JCS re-emits them as \uXXXX with lowercase hex.
    let produced = canonicalize("{\"k\":\"\\u001F\"}").expect("canonicalize");
    assert_eq!(produced, "{\"k\":\"\\u001f\"}");
}
