# jcs-canonicalize

A CLI tool and small Rust library for [RFC 8785](https://datatracker.ietf.org/doc/html/rfc8785) JSON canonicalization (JCS).

RFC 8785 defines a deterministic byte representation for JSON values: the same input always produces the same bytes, regardless of key order, whitespace, or number formatting. That property is what lets a cryptographic signature over those bytes verify reliably across implementations, languages, and time.

## What this is, and what it is not

This crate is a thin packaging on top of [`serde_jcs`](https://crates.io/crates/serde_jcs) (the de facto JCS implementation in Rust). It adds:

- A standalone `jcs-canonicalize` CLI binary suitable for shell pipelines and packaged distribution (crates.io, nixpkgs).
- A `sha256_jcs_hex` helper for the common "canonicalize then SHA-256" pattern used in signed evidence and content-addressed stores.
- A conformance test suite that exercises the [cyberphone/json-canonicalization](https://github.com/cyberphone/json-canonicalization) reference corpus (vendored under `tests/fixtures/cyberphone-corpus/`, Apache-2.0), the RFC 8785 Appendix-E-style examples, and targeted edge cases (non-BMP UTF-16 key sort, ECMAScript `Number.toString` boundaries, C0 control escaping, NaN/Infinity rejection).

The actual canonicalization engine — UTF-16-ordered keys, ryu-js number formatting, escape rules — lives in `serde_jcs`. The correctness of this crate is the correctness of `serde_jcs`, verified by the test suite below.

## Alternatives

If you do not need the CLI, consider depending on one of these libraries directly:

- [`serde_jcs`](https://crates.io/crates/serde_jcs) — what this crate wraps. The most-downloaded RFC 8785 implementation on crates.io.
- [`canon-json`](https://crates.io/crates/canon-json) — RFC 8785 as a `serde_json` Formatter, maintained by the [`containers`](https://github.com/containers) organization.
- [`json-canon`](https://crates.io/crates/json-canon) — older, stable RFC 8785 serializer.

The reference implementation, used to generate the conformance test vectors, is [cyberphone/json-canonicalization](https://github.com/cyberphone/json-canonicalization) (Go, Java, JavaScript, Python, .NET).

## CLI

Reads JSON from stdin, writes JCS-canonical JSON to stdout. Exit codes: `0` on success, `1` on parse or canonicalize error, `2` on I/O error.

```sh
echo '{"b":2,"a":1}' | jcs-canonicalize
# {"a":1,"b":2}
```

Canonicalizing canonical output is a fixed point — running the CLI twice produces the same bytes.

## Library

```rust
pub fn canonicalize(input: &str) -> anyhow::Result<String>;
pub fn sha256_jcs_hex<T: Serialize>(value: &T) -> anyhow::Result<String>;
```

`canonicalize` takes a JSON string and returns its RFC 8785 canonical form. `sha256_jcs_hex` takes any `serde::Serialize` value and returns the lowercase-hex SHA-256 of its JCS-canonical bytes.

```rust
use jcs_canonicalize::{canonicalize, sha256_jcs_hex};

let canonical = canonicalize(r#"{"b":2,"a":1}"#)?;
assert_eq!(canonical, r#"{"a":1,"b":2}"#);

#[derive(serde::Serialize)]
struct Evidence { host: String, ok: bool }
let digest = sha256_jcs_hex(&Evidence { host: "host-01".into(), ok: true })?;
// 64-char lowercase hex SHA-256
```

## Conformance and trust

`cargo test` runs every contract this crate claims to honor:

| Test file | What it asserts |
|---|---|
| `tests/cyberphone_corpus.rs` | All six input/output pairs from the cyberphone reference corpus produce byte-identical canonical output, and every corpus input is a fixed point under re-canonicalization. |
| `tests/edge_cases.rs` | Non-BMP key sort by UTF-16 code units (not UTF-8 bytes); ECMAScript `Number.toString` boundaries (1e21, 1e-7, 1e20, trailing-zero trimming); negative zero; NaN/Infinity rejection; C0 control escape vs U+007F literal. |
| `tests/jcs_golden.rs` | A pinned golden input/output pair, an idempotence check, key-reordering invariance, and invalid-JSON rejection. Drift in the golden bytes = signature contract broken. |

If any test in `tests/cyberphone_corpus.rs` fails after a dependency update, **do not release**: the canonicalization bytes have changed and every signature ever produced over the previous bytes is invalidated.

### Precision contract

RFC 8785 §3.2.2.3 defines numbers to be IEEE 754 double precision. Integers above 2^53 (or below -2^53) are silently rounded to the nearest representable double on the way in. If your input includes such integers, encode them as JSON strings, not as JSON numbers.

### Duplicate-key contract

RFC 8785 §3.2.3 leaves duplicate object keys as undefined behavior. This crate, via the underlying `serde_json` parser and `serde_jcs` serializer, resolves duplicates by **last-value-wins**: `{"a":1,"a":2}` canonicalizes to `{"a":2}` with no warning. If duplicate-key rejection matters to your threat model, validate the input separately before canonicalizing.

## Use cases

Anywhere a signature over JSON needs to verify across implementations or across time:

- **Signed audit evidence** in compliance and attestation workflows.
- **JWS detached signatures** with JSON payloads where key ordering must be deterministic.
- **Supply-chain attestations** (sigstore, in-toto adjacent) where the artifact under signature is a JSON document.
- **Content-addressed JSON stores** where the canonical form is what gets hashed for identity.
- **Cross-language signature workflows** where one side writes JSON and another verifies, and both need to agree on the byte sequence under signature.

## Installation

### From crates.io

```sh
cargo install jcs-canonicalize
```

### Nix

```sh
nix run nixpkgs#jcs-canonicalize -- < input.json
```

## Origin

Extracted from [arcanesys/nixfleet](https://github.com/arcanesys/nixfleet), where it serves the signed compliance-evidence chain for regulated NixOS estates. The functionality is generic RFC 8785; the NixFleet consumption is one use case among many.

## License

MIT. See `LICENSE`. The vendored cyberphone corpus under `tests/fixtures/cyberphone-corpus/` is Apache-2.0; see the `LICENSE` and `NOTICE.md` files in that directory.
