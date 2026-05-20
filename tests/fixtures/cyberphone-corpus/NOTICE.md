# Cyberphone JCS conformance corpus (vendored)

The files in `input/` and `output/` in this directory are copied verbatim
from the reference RFC 8785 implementation repository:

  https://github.com/cyberphone/json-canonicalization
  Copyright 2018 Anders Rundgren

They are distributed under the Apache License, Version 2.0; see the
`LICENSE` file in this directory for the full text.

Each `input/<name>.json` is a JSON document. The corresponding
`output/<name>.json` is the expected RFC 8785 canonical form of that
document. A conformant canonicalizer must produce byte-identical output
for every pair. The `tests/cyberphone_corpus.rs` test enforces that.

This vendoring is for offline conformance testing only. No modification
has been made to the files.
