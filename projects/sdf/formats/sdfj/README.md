# sdfj

Core Rust types for the SDF Json (`.sdfj` / `.sdfjz`) format. A document
records a model's calls as JSON tables whose entries reference each other by
index.

The types are the data model, with optional `serde` support behind the default
`serde` feature. A parse checks the keys and kinds each entry holds:

- a document has to hold `version`
- a missing table or names map reads empty
- an unknown key fails to read
- an entry's `kind` picks its variant, and an optional key the entry leaves
  out reads `None`
- an optional key holding `null` fails to read
- an index reads only as a whole number

A parse leaves the references to the reader. A write errors on a NaN, an
infinity, or a repeated object key. `SDFJ_VERSION` holds the version a
document's `version` reads.

See [`docs/sdf-json-file-format.md`](docs/sdf-json-file-format.md) for the
format specification.
