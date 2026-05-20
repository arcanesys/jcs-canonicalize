# jcs-canonicalize

A CLI tool and Rust library for [RFC 8785](https://datatracker.ietf.org/doc/html/rfc8785) JSON canonicalization (JCS).

RFC 8785 defines a deterministic byte representation for JSON values: the same input always produces the same bytes, regardless of key order, whitespace, or number formatting. That property is what lets a cryptographic signature over those bytes verify reliably across implementations, languages, and time.

## CLI

Reads JSON from stdin, writes JCS-canonical JSON to stdout. Exit codes: `0` on success, `1` on parse or canonicalize error, `2` on I/O error.

```sh
echo '{"b":2,"a":1}' | jcs-canonicalize
# {"a":1,"b":2}
```

Round-trip is a fixed point: canonicalizing canonical output yields the same bytes.

## Library

Two public functions:

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

## Conformance

The repository ships:

- An RFC 8785 Appendix E conformance test suite (`tests/rfc8785_appendix_e.rs`).
- A golden-file test (`tests/jcs_golden.rs`) that asserts byte-identical output across releases. Drift in this file means the canonicalization contract changed and existing signatures over old outputs no longer verify.
- A library-level idempotence test: `canonicalize(canonicalize(x)) == canonicalize(x)`.

`cargo test` runs the full suite (10 tests).

## Use cases

Anywhere a signature over JSON needs to verify across implementations or across time. Examples:

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

(Pending nixpkgs upstream merge.)

## Origin

Extracted from [arcanesys/nixfleet](https://github.com/arcanesys/nixfleet), where it serves the signed compliance evidence chain for regulated NixOS estates. The functionality is generic RFC 8785; the NixFleet consumption is one use case among many.

## License

MIT. See `LICENSE`.
