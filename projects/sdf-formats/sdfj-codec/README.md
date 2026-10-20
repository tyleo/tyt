# sdfj-codec

Reads and writes SDF Json `.sdfj` documents over the data types the `sdfj`
crate defines.

- `from_sdfj_file_bytes`: `.sdfj` JSON bytes into an `SdfjFile`
- `to_sdfj_file_bytes` / `to_sdfj_pretty_file_bytes`: an `SdfjFile` to compact
  or pretty-printed JSON with a trailing newline

Each transcodes the JSON through the `DecodeSdfjJson` and `EncodeSdfjJson`
traits. A caller implements the traits or takes `DependenciesImpl` behind the
default `impl` feature, which codes the JSON with exact float
parsing and the key order kept. A write errors when the file holds a NaN, an
infinity, or a repeated object key.
