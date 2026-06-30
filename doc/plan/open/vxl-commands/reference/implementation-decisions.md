# Implementation decisions

_Part of the [Vxl Command-Line Reference](../README.md)._

Code-level decisions made while building these commands, recorded as they land.
Command-design rationale lives in [design notes](design-notes.md); this log is
for implementation choices a reviewer of the Rust would want explained.

## MeshFormat

`MeshFormat` carries the two glTF variants, `Gltf` (`.gltf`) and `Glb` (`.glb`),
which are the only mesh formats `mesh` and `voxelize` handle for now. glTF was
chosen over `fbx` and `obj` because it has a mature pure-Rust reader, the `gltf`
crate, so the mesh I/O needs no Blender shell-out or C++ bindings. `from_path`
infers the variant from the extension, mirroring `Format::from_path`. Other
formats are stated future work in the [design notes](design-notes.md), not a
current variant.

No `extension()` method yet. `Format` carries none, and the only caller is the
defaulted output path, which lands with the `mesh` command.

## --select-index parser

`FromStr::Err` is `String`. That is the idiomatic error for a clap value parser
because `String` converts into the boxed error clap expects, and it avoids a
one-off error type. The clap wiring puts `Vec<SelectIndex>` in an `#[arg]`, which
is first compiled when the `mesh` command lands, so the bound is exercised then.

The public API is just `contains`. The union resolver tests each object index
against every selector, which is all `contains` needs to support. Validating an
index against the real object count is resolution-time work and lands with the
shared selector resolver.

## --select path glob

Matching delegates to globset, the project's standard glob engine, through a
`Dependencies::match_glob` method. It is the same `GlobBuilder` with
`literal_separator(true)` that `tyt-injection` exposes, reimplemented inline in
vxl's `implementation/` behind the `impl` feature so vxl gets the standard glob
behavior without depending on `tyt-injection`. globset is an optional dependency
gated by `impl`, beside the codec crates. A hand-rolled matcher was tried first
but dropped, since reproducing globset by hand risks subtle divergence from the
shared format.

`PathGlob` carries only the pattern, `**/`-prepended unless it already starts
with `**/`, the same normalization the tyt hierarchy commands apply. It hands
that normalized string to `match_glob` rather than matching itself, because the
globset engine lives behind `impl`. Expanding a matched node's subtree and
unioning the selectors is resolution-time work that lands with the commands.

## Material packing model

`ChannelSource` holds an attribute key as a `String` rather than a fixed enum,
because the voxj format stores attributes generically and a packing may read any
attribute, not only the recommended ones. `smoothness` is not a stored
attribute; the parser canonicalizes `smoothness` and `1-smoothness` to the
`roughness` attribute with the matching inversion, so the bake always reads one
real attribute.

`ChannelPacking` sizes the image by the highest channel letter present, so
naming `A` makes a four-channel image even when `G` and `B` are unnamed, which
then default to `0`. This is what lets `R=metallic,A=smoothness` reproduce the
metallic-smoothness preset. The docs' "number of channels named" holds for
contiguous packings, the common case.

The `--texture` presets lower to `TextureBake` via `Texture::bake`. All but
`albedo` are scalar `ChannelPacking`s built the same way `--texture-map` parses,
so a preset and its channel-list spelling are the same value. `albedo` is the
RGBA base color, which a scalar packing cannot express, so it is its own
`TextureBake` variant. `computed-occlusion` lowers to a packing whose one source
is `ComputedOcclusion`, which the bake treats as forcing an unwrap layout, so no
separate flag is needed.

## Custom attribute bindings

`ChannelSource::Attribute` grew a `component: Option<ColorComponent>`. The parser
reads a trailing single-letter `.r`/`.g`/`.b`/`.a` as the component and treats a
longer dotted suffix as part of the key, so an attribute key that contains a dot
still parses whole. `key` stays the name token as written; whether it is a
`--define-attribute` binding alias or a direct attribute key is resolved later
against the binding table, which lands with the bake, the same deferral as the
selector resolvers. Validating a component against the resolved type, a component
only on a color and a color only with a component, is that same resolution-time
work, so the parser does not enforce it yet.

`AttributeType` is a `ValueEnum` of `scalar` and `color`; `AttributeBinding`
holds the name, palette index, key, and type. The `--define-attribute` flag is a
repeatable, three-or-four-value option, which clap groups per occurrence only at
fixed arity, so the clap wiring is deferred to the `mesh` command beside the
other unbuilt map flags. `AttributeBinding::FromStr` parses the whitespace
`name palette key [type]` form for now, which also drives the unit tests; the
fixed-versus-variable arity choice is made when `mesh` lands.

`smoothness` still canonicalizes to inverted `roughness` in the parser, before
the binding table exists, so a binding that shadows `roughness` also flows
through `smoothness`. A binding literally named `smoothness` would be bypassed, a
niche the docs do not promise.

## Shared voxj encoding options

`VoxjEncodingOptions` is a `clap::Args` group with the four flags that shape a
voxj document: `--format`, `--encoding-preset`, `--position-encoding`, and
`--sample-encoding`, with `encoding` and `resolve_format` resolution methods.
Both `to voxj` and `voxelize` flatten it; `to voxj` adds `--ext` and
`--edit-state`, which a voxelized mesh has no source for, so they are not in the
shared group. clap cannot prefix a flattened group, so sharing happens at this
sub-group rather than wrapping each command's whole writer surface. Output-path
defaulting stays per command, since the default stem differs: `to voxj` defaults
from the voxel file, `voxelize` from the mesh.

`--optimize` was renamed to `--encoding-preset`, and the type `VoxjOptimize` to
`VoxjEncodingPreset`, because its values are `size`, `fast`, and `pretty`, and
`pretty` is not an optimization. "Encoding preset" describes all three and reads
clearly beside the per-block `--position-encoding` and `--sample-encoding`.

## palette command group

`tyt meta create-command` scaffolds the `palette` group. Its leaf prints a
placeholder line for now; the real `list`, `show`, and `quantize` subcommands
land later. The scaffolder's command template calls `Dependencies::write_stdout`,
a method every freshly generated crate's trait carries, so restoring that method
to vxl was the only change the generated code needed to compile.

vxl implements `write_stdout` against `std::io::stdout().write_all` as a free
function in the implementation module rather than delegating to
`tyt_injection::write_stdout` the way the prefixed tyt crates do, because vxl
stays independent of the tyt support crates.

## palette show

The command splits `--attribute` into a key and an optional trailing color
component with its own parser rather than reusing `ChannelSource`, because
`--attribute` takes only `<key>.component` and must reject the `1-` inversion,
the `0` and `1` constants, and the `smoothness` alias a `--texture-map` channel
accepts. The split rule itself is the shared one: a trailing single letter that
names a color component is the component, and a longer dotted suffix stays part
of the key, so a dotted attribute key still parses whole.

`Dependencies::palette_show` carries the load, render, and print together, like
the `to_*` converters, so the command struct only parses flags. Its core is a
pure function over a `VoxPalette` that returns the output string, which keeps the
formatting unit-testable without the filesystem; the wrapper loads the document,
selects the palette by index, and writes the result to stdout.

The value type resolves at render time, not parse time, the same deferral the
selectors use: the type is inferred from the first concrete value unless `--type`
overrides it, and a component on a scalar is rejected once the type is known. A
value that does not match the resolved type falls back to its raw text rather
than failing, so a forced `--type` never crashes on a stray cell.

JSON is hand-rolled rather than pulling in a serializer, since vxl carries no
serde dependency and the shape is a flat object of the palette index, the
attribute label, and the rendered values. The values carry their own JSON type,
a hex string for a color and a number otherwise, so no separate type field is
reported. The richer JSON forms are left to the V2 follow-ups in
[palette show](palette/show.md).

## voxelize

The mesh-reading and rasterization live in voxsmith, not vxl, so the command
follows the `to voxj` template: `commands/voxelize.rs` parses the flags and
`implementation/voxelize.rs` calls voxsmith, which returns a
[`VoxMain`](voxcore::VoxMain) that the same `VoxjFileBuilder` path then encodes
with the shared `--format` / `--encoding-preset` / `--position-encoding` /
`--sample-encoding` options. vxl gains no mesh dependency of its own; the `gltf`
crate and the voxelizer sit behind voxsmith.

voxsmith gets a new Cargo feature (a `gltf` feature beside the existing
`goxl` / `mvox` / `qbcl` / `vmax` / `voxj` features) that gates the `gltf`
dependency and the voxelization module. Unlike the codec features it is not a
load/save converter, so it does not turn on `_codec`; it is a mesh-to-`VoxMain`
front end. vxl's `impl` feature enables it. The voxsmith entry point takes the
glTF bytes, the grid resolution, the fill mode, and the fill color, and returns a
`VoxMain` carrying one object placed by one root node. When the caller resolved
the grid from `--scale`, it sets that node's transform scale to `<meters>` so the
assembled model keeps its source size; `--side-length` leaves the scale at `1`.

Grid resolution is resolved in vxl before the call, into a single voxel-count
triple, so voxsmith takes counts and never re-reads the mesh extent: `--side-length`
caps the longest axis and sizes the others to preserve aspect, while `--scale`
divides each meter extent by `<meters>` and rounds up. The mutual exclusion of
`--side-length` and `--scale` is a clap `ArgGroup` with `required = true`, so
exactly one is always present.

`--fill-mode solid` paints every voxel the one `--fill-color`, so the document
has one palette with a single `rgba` cell. `--fill-mode surface` samples each
voxel's color from the glTF material, so its palette holds one cell per distinct
sampled color; `--fill-color` is rejected with `surface`. The solid flat-color
path is the default and the MVP; the surface color-sampling path can land after
it, with surface initially falling back to the flat color until sampling exists.
`--fill-color` accepts a `#RRGGBBAA` hex or the name `white`, parsed in vxl into
the `rgba` value voxsmith stores.
