# meshconv

Reads and writes mesh file formats through the meshdoc state. Each format's
`-meshdoc` bridge crate owns its conversion. This crate picks the bridge for
a format and moves a document's files through it.

## Formats

A format is a marker type implementing `Format`, one per enabled format
feature: `Gltf` (`.glb`, `.gltf`). The trait carries the format's name,
extensions, package flag, writer options type, the loose files a primary
references, and the bare read and write over its bridge. `ReadFormat` lists
the formats a document can be read as, one variant per marker. `WriteFormat`
lists the write targets with their options. glTF carries a
`gltf::GltfWriteFormat`: a `GltfContainer` picking `.glb` or `.gltf` and the
`GltfWriteOptions` for where the images go, as loaded, embedded, or loose.
This crate re-exports the options from the bridge. Each format feature opens
a module of its name: `gltf` holds the marker and the format's writer
options.

`ReadFormat::with` and `WriteFormat::with` turn a runtime format into its
marker. Each runs a visitor with the marker as a type parameter, so a
function over a format is one generic body. The match over the variants
lives in `with` alone. `ReadFormat::ALL` lists the enabled formats.

## Files

A document is a list of `MeshDocumentFile`. Each holds a document-relative
path and its bytes. The primary file sits at the empty path. A `.gltf` that
references loose files beside it holds one entry per loose file at the
relative path the primary references it by. A `.glb`, or a `.gltf` with
data URIs, is one entry.

## Conversion

These functions move a document through a bridge as a bare `MeshMain<()>`:

- `read_document_files` / `write_document_files`: between a path on disk and
  a document's files, through the caller's `ReadFile`, `ListDir`, and
  `WriteFile`. A read takes the primary, asks the format which loose files
  it references, and reads those beside it. A write puts a loose file
  beside the output.
- `read` / `write`: between a document's files and a bare `MeshMain`. A read
  takes the format's ext off. A write hands the state to the bridge's
  `to_x_mesh_main`, which gives it a synthesized ext. The bridge's one typed
  writer then writes it.
- `load` / `save`: `read_document_files` then `read`, and `write` then
  `write_document_files`.
- `check_document_files`: a `MeshCheck` for whether the files decode, then
  one per spec check the format defines.

Each takes a `Dependencies`: any type implementing each enabled format's
dependencies trait, such as `gltf::GltfDependencies`, which returns that
format's bridge dependencies. A type holding those in another value
implements `ForwardDependencies` instead and gets every trait from that one
impl. A format feature enables its bridge's `codec` feature.
`DependenciesImpl`, behind the `impl` feature, binds std's filesystem and
each format's bridge impl.

## The Ext

A format's ext is the state its bridge keeps so a loaded file writes back
exactly. The `ext` feature, on by default, opens the `ext` module, where the
typed pairs carry that state boxed in a `MeshconvMeshMain`, a
`MeshMain<Box<dyn MeshconvExt>>`. `FormatExt` extends each format with its
typed path. `MeshconvExt` is any `MeshExt` that can downcast and clone:

- `read_with_ext` / `write_with_ext`: a read boxes the ext the bridge loads.
  A write downcasts the box to the format's ext. A box holding another
  format's ext writes the synthesized file the bare pair writes.
- `load_with_ext` / `save_with_ext`: the same from and to a path.

The `ext` feature enables each bridge's `serde` feature.
