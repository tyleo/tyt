# sdfconv

Reads and writes SDF file formats through the sdfcore state. Each format's
`-sdfcore` bridge crate owns its conversion. This crate picks the bridge for a
format and moves a document's bytes through it.

## Formats

A format is a marker type implementing `Format`, one per enabled format
feature: `Sdfj` (`.sdfj`, `.sdfjz`). The trait carries the format's name,
extensions, writer options type, and the read and write over its bridge.
`ReadFormat` lists the formats a document can be read as, one variant per
marker. `WriteFormat` lists the write targets with their options. SDF Json
carries an `sdfj::SdfjSerialization` picking the container. A read takes either
container.

`ReadFormat::with` and `WriteFormat::with` turn a runtime format into its
marker. Each runs a visitor with the marker as a type parameter, so a function
over a format is one generic body. `ReadFormat::ALL` lists the enabled
formats.

## Conversion

- `read` / `write`: between a document's bytes and an `SdfMain`
- `load` / `save`: the same from and to a path, through the caller's
  `ReadFile` and `WriteFile`

`load` takes `LoadOptions`. With `fonts` set, the format's codec also loads
the document's fonts into the `SdfMain`.

Each takes a `Dependencies`: any type implementing each enabled format's
dependencies trait, such as `sdfj::SdfjDependencies`. A type holding those in
another value implements `ForwardDependencies` instead and gets every trait
from that one impl. `DependenciesImpl`, behind the `impl` feature, binds std's
filesystem and each format's bridge impl.
