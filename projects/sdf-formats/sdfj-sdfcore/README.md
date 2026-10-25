# sdfj-sdfcore

Converts between SDF Json documents and an sdfcore main. The `sdfj` crate
defines the document types. This crate carries a parsed document into
sdfcore's `SdfMain` and back.

## Document conversion

- `from_sdfj_file`: a parsed `SdfjFile` into an `SdfMain`. A load errors on a
  version other than `SDFJ_VERSION`, an index past the id space, a step holding
  both or neither of `material` and `pattern`, and a state `SdfMain::new`
  rejects. An error about one entry starts with the entry's place, such as
  `steps[3]`.
- `to_sdfj_file`: an `SdfMain` into an `SdfjFile`

Each entry keeps its place in its table, so a document reads back as the same
state.

## Bytes conversion

The `codec` module, behind the default `codec` feature, goes straight to and
from file bytes over `sdfj-codec`:

- `codec::from_sdfj_bytes`: `.sdfj` or `.sdfjz` bytes into an `SdfMain`,
  detecting the form by its leading bytes
- `codec::from_sdfj_bytes_with_fonts`: the same, with the document's fonts
  loaded
- `codec::to_sdfj_bytes` / `codec::to_sdfj_pretty_bytes`: an `SdfMain` to
  compact or pretty-printed `.sdfj` JSON
- `codec::to_sdfjz_bytes`: an `SdfMain` to a `.sdfjz` zip archive

Each takes the codec's dependencies, which `sdfj_codec::DependenciesImpl`
supplies.
