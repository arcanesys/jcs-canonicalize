//! Conformance against the cyberphone/json-canonicalization reference corpus
//! (vendored at tests/fixtures/cyberphone-corpus/). The reference corpus is
//! the de facto interoperability benchmark for RFC 8785 implementations.
//!
//! See tests/fixtures/cyberphone-corpus/NOTICE.md for attribution.

use jcs_canonicalize::canonicalize;

const CASES: &[(&str, &str, &str)] = &[
    (
        "arrays",
        include_str!("fixtures/cyberphone-corpus/input/arrays.json"),
        include_str!("fixtures/cyberphone-corpus/output/arrays.json"),
    ),
    (
        "french",
        include_str!("fixtures/cyberphone-corpus/input/french.json"),
        include_str!("fixtures/cyberphone-corpus/output/french.json"),
    ),
    (
        "structures",
        include_str!("fixtures/cyberphone-corpus/input/structures.json"),
        include_str!("fixtures/cyberphone-corpus/output/structures.json"),
    ),
    (
        "unicode",
        include_str!("fixtures/cyberphone-corpus/input/unicode.json"),
        include_str!("fixtures/cyberphone-corpus/output/unicode.json"),
    ),
    (
        "values",
        include_str!("fixtures/cyberphone-corpus/input/values.json"),
        include_str!("fixtures/cyberphone-corpus/output/values.json"),
    ),
    (
        "weird",
        include_str!("fixtures/cyberphone-corpus/input/weird.json"),
        include_str!("fixtures/cyberphone-corpus/output/weird.json"),
    ),
];

/// Each cyberphone input must produce byte-identical canonical output.
/// The reference outputs include a trailing newline; we trim it before
/// comparison so the assertion targets the canonical bytes themselves.
#[test]
fn cyberphone_reference_corpus_byte_identical() {
    let mut failures = Vec::new();
    for (name, input, expected_raw) in CASES {
        let expected = expected_raw.strip_suffix('\n').unwrap_or(expected_raw);
        let produced = match canonicalize(input) {
            Ok(p) => p,
            Err(err) => {
                failures.push(format!("{name}: canonicalize errored: {err:?}"));
                continue;
            }
        };
        if produced != expected {
            failures.push(format!(
                "{name}:\n  produced ({pl} bytes) = {produced}\n  expected ({el} bytes) = {expected}",
                pl = produced.len(),
                el = expected.len(),
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{n} cyberphone corpus case(s) failed:\n{}",
        failures.join("\n"),
        n = failures.len()
    );
}

/// Canonical output is a fixed point under re-canonicalization for every
/// case in the reference corpus.
#[test]
fn cyberphone_reference_corpus_idempotent() {
    for (name, input, _) in CASES {
        let once = canonicalize(input).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let twice = canonicalize(&once).unwrap_or_else(|e| panic!("{name} (twice): {e:?}"));
        assert_eq!(once, twice, "{name}: canonicalization is not idempotent");
    }
}
