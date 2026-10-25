# sdfj-codec

Reads and writes SDF Json `.sdfj` / `.sdfjz` documents over the data types
the `sdfj` crate defines.

- `from_sdfj_file_bytes`: `.sdfj` JSON bytes into an `SdfjFile`
- `to_sdfj_file_bytes` / `to_sdfj_pretty_file_bytes`: an `SdfjFile` to compact
  or pretty-printed JSON with a trailing newline
- `from_sdfjz_file_bytes` / `to_sdfjz_file_bytes`: the zip-packaged `.sdfjz`
  form
- `from_sdfj_or_sdfjz_file_bytes`: either form, detected by its leading bytes
- `wrap_sdfjz` / `unwrap_sdfjz`: `.sdfj` bytes into a `.sdfjz` archive and
  back, leaving the JSON as it is
- `load_sdfj_fonts`: the glyphs of each font an `SdfjFile` references, for the
  characters of the texts reading the font

The JSON coding, the `.sdfjz` member's deflate, and the font lookup and
decoding come in through the `dependencies` traits. A caller implements the
traits or takes `DependenciesImpl` behind the default `impl` feature, which
codes the JSON with exact float parsing and the key order kept. It finds fonts
among the system's installed faces and decodes TrueType and OpenType glyphs. A
write errors when the file holds a NaN, an infinity, or a repeated object key.
