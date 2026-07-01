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
front end gated by `dep:gltf` plus the `import` and `utils` gltf features (which
pull `gltf::import_slice` and the buffer `Reader`). vxl's `impl` feature enables
it through `voxsmith/gltf`. voxsmith exposes two free functions in
`convert/gltf/`: `gltf_mesh_extent` returns the mesh's meter extent so vxl can
size the grid, and `voxelize_gltf` takes the byte slice, the resolved voxel
counts, the fill mode, the fill color, and the node scale, returning a `VoxMain`
of one object placed by one root node. When the caller resolved the grid from
`--scale` it passes `<meters>` as the node scale so the assembled model keeps its
source size; `--side-length` passes `1`. The two functions each call
`import_slice`, so the glTF parses twice (extent, then rasterize); the file is
small for voxelization and this keeps the resolution policy in vxl with voxsmith
taking plain counts. `import_slice` auto-detects `.glb` versus `.gltf` from the
bytes, so `--from` is accepted for the documented interface but does not steer
parsing; a `.gltf` with external `.bin` buffers cannot resolve from bytes and
errors.

Grid resolution is resolved in vxl's `implementation/voxelize.rs` before the
voxsmith call, into a single voxel-count triple: `--side-length` caps the longest
axis and sizes the others to preserve aspect, while `--scale` divides each meter
extent by `<meters>` and rounds up. The mutual exclusion of `--side-length` and
`--scale` is a clap `ArgGroup` with `required = true`, so exactly one is present.
The voxj writer is the same `VoxjFileBuilder` path, factored into a shared
`implementation/write_voxj_document` helper that `to voxj` also uses; voxelize
calls it with `ext = false` and `EditStateMode::Never`, since a voxelized mesh
carries neither a source `ext` block nor an editor build volume.

The voxelizer rasterizes with a separating-axis triangle-vs-voxel-box test in
grid space (each voxel a unit cube), and `solid` adds a six-connected flood fill
from the grid boundary so every cell the outside cannot reach is filled. Working
in grid space keeps the box a unit cube even when the per-axis voxel size differs,
which is valid because the map onto grid space is affine and affine maps preserve
overlap.

Coordinates convert from glTF's Y-up to Voxel Json's Z-up: each gathered
world-space point is sent through `(x, y, z) -> (x, -z, y)`, a +90 degree
rotation about X that preserves the right-handedness both formats use, so a model
stands upright in a Z-up editor. The conversion is fixed by the two formats'
specs (glTF mandates Y-up), so it needs no flag; a future `--axes` style override
for the rare mis-authored file is left as future work, and `vxl mesh` (the
inverse) must mirror this mapping.

`--fill-mode solid` paints every voxel the one `--fill-color`, so the document
has one palette with a single `rgba` cell. `--fill-mode surface` is a hollow
shell and `--fill-color` is rejected with it; sampling each voxel's color from the
glTF material is future work, so for now `surface` falls back to the flat color
(the `white` default). `--fill-color` accepts a `#RRGGBBAA` or `#RRGGBB` hex or a
color name, parsed in vxl into the `rgba` value voxsmith stores. Reading the mesh
fails the conversion as a `voxsmith::Error::Gltf`, a new variant beside the codec
ones.

The richer color path is now designed though unbuilt: `--material-mode`
(`auto` / `per-primitive` / `per-texel` / `flat`), a `--max-palette` cap reusing
the `palette quantize` reduction engine, and the material-follows-color rule
shared with quantize and remap. See [voxelize](voxelize.md) and the
[design notes](design-notes.md). The shipped flat-color path above becomes
`--material-mode flat`, and the `--fill-color`/`--fill-mode surface` guard is
replaced when that work lands.

## palette show V2

The V1 `--index`, `--attribute`, `--type`, and `--format` flags become one
repeatable `--attribute <palette> <attribute> <format>` selector and a global
`--layout`. `--json` is gone; its `compact` and `pretty` renderings are the
`compact-json` and `pretty-json` values of `--layout`, beside `row`,
`row-no-header`, `column`, `column-no-header`, and a `markdown` table. The same
`--format`-to-`--layout` move lands on the other read commands so the json forms
read the same everywhere; `--layout` defaults to a flat `row` rather than the
spec's earlier row-for-one, table-for-several default, which the explicit values
made redundant.

`row` pads only the header, to the longest, so the first value of each row lines
up; the values themselves are not column-aligned, since aligning every value
would let one wide attribute like a long float stretch the rest. The rows are
separated by a blank line. A swatch row's cells abut into a strip, except a value
with no swatch, a bool that fell back to raw text, which keeps the one space so it
does not run together; the other formats always space their cells. The
`-no-header` variants of `row` and `column` reuse the same renderer with the
header column or row dropped, and a header-less column sizes its width from the
cells alone.

clap derive has no typed grouping for several values per option occurrence, so
`--attribute` is a `Vec<String>` with `num_args = 3` and `ArgAction::Append`,
chunked by three in the command. The fixed arity makes the flattened list chunk
unambiguously, the same reason the design fixes three fields. The fields use
`value_names` so help names each, the one place a multi-value option needs the
plural of the `value_name` rule. The format field is parsed with
`ValueEnum::from_str` so its error lists the variants.

The selector lowers to three utility types, `PaletteRef`, `AttributeRef`, and
`AttributeSelector`, with parsing on the types and resolution in the
implementation, the same split the channel and binding types use. `AttributeRef`
carries the `<key>.component` split moved off the old command-level
`parse_attribute`, so the dotted-key rule lives with the type. `PaletteShowFormat`
gains `PartialEq` for the selector's derive. The new layout enum is
`PaletteShowLayout`.

`--type` is gone; the type is always inferred from the cells. `AttributeType`
stays, because the inference and the `--define-attribute` binding still use it. A
color component is now a `0..255` byte rather than the V1 `0..1` fraction, a new
`Sample::Component(u8)` distinct from a scalar; a scalar still maps its `0..1`
value onto a gray level, while a component is its byte directly.

Resolution skips a `*`-matched palette that lacks a named attribute but errors on
a named palette that lacks it, so a typo on a named palette is caught while a
broad `'*'` stays quiet. A color component on an inferred scalar is an error
whether the palette was named or `*`-matched, since it names a channel the value
does not carry. Collections come out in selector order, then palette then
attribute order, which is the JSON render order too.

Alignment measures a cell's visible width past its ANSI CSI escapes, so the
zero-width swatch codes do not throw the value columns off. `render` is pure over
the resolved collections and `resolve_collections` pure over the loaded state, so
both unit-test without the filesystem.

The `--json`-to-`--layout` move has since landed across the read reports, with a
shared `ReportLayout` of `markdown`, `pretty-json`, and `compact-json`. `palette
show` keeps its own richer `PaletteShowLayout`, since it adds the `row`, `column`,
and swatch arrangements, but its `markdown`, `pretty-json`, and `compact-json`
names and behavior match the shared form. The bare palette `--from palette` input
and a shared JSON envelope stay deferred.

JSON is built with `serde_json`, the crate `info` and `validate` already pull in,
replacing the V1 hand-rolled serializer: `compact-json` is `to_string` and
`pretty-json` is `to_string_pretty`, the same indented form `info` emits, so the
read commands' JSON reads alike. An integral scalar still serializes as an integer
so it matches the text layouts. The markdown layout shares one `markdown_table`
helper with `info`, and the `visible_width` and `pad_right` primitives move to a
`text_width` module; the shared table measures past ANSI escapes so swatch cells
align, which `info`'s plain text neither needs nor is hurt by.

`--width` wraps the `row` layouts so a 255-entry palette folds into a block
instead of one multi-thousand-column line that the terminal mangles when it
wraps. It is a `Width` of `terminal`, `unlimited`, or a column count, parsed by
`FromStr` since the count rules out a `ValueEnum`. The default is `terminal`,
resolved against the terminal width read from stdout; a non-terminal stdout, as
when piped, resolves to no wrapping so a pager or file keeps the full line. The
width is read with a `libc` ioctl rather than the `terminal_size` crate, since
`libc` is already in the lock and the crate is not, so the ioctl adds no new
download; it is gated to unix, and other platforms simply do not wrap. Only the
`row` layouts wrap, as `column` and `markdown` are already vertical or
self-sizing.

## Parent-prefixed command names

`create-command` lays every command, group or leaf, in one flat `src/commands/`
namespace flattened through `pub use {snake}::*`, so a leaf's CLI name had to be
unique across the whole crate at both the file (`show.rs`) and the type (`Show`)
level. That blocked `vxl hierarchy show` while `vxl palette show` already owned
both. Now a grouped command's type and file name carry their parent-group path:
`show` under `palette` is the `PaletteShow` type in `palette_show.rs`, and the
same `show` under a `hierarchy` group would land as `HierarchyShow` in
`hierarchy_show.rs`. The clap
`#[command(name = "show")]` keeps the bare CLI name, so the surface is unchanged
and both still run as `... show`. Top-level commands and the group structs take
no prefix, so `info`, `palette`, and `to` are untouched.

The prefix keeps the namespace flat instead of nesting each group in its own
`commands/<group>/` submodule. Flat naming leaves the `mod.rs` shape and the enum
imports as plain `use crate::commands::{...}`, so the generator's string inserters
and templates feed the prefixed names with no structural change. The existing
`palette` and `to` leaves were renamed to the same rule, `Show` to `PaletteShow`
and `Goxl` to `ToGoxl` and so on, so generated and hand-written commands share one
layout.

## hierarchy show

This is built in three iterations. The first landed the core tree with its
instancing and unplaced marks and `--collapse-instances`; the second the
`pattern` glob with `--collapse-ancestors`/`--collapse-descendants`; the
`--show-transforms`/`--show-bounds`/`--show-extents` subtrees follow.

`Dependencies::hierarchy_show` carries the load, render, and print together like
the other read reports, so the command struct only parses flags. Its core is a
pure `render` over a `VoxMain` that returns the output string, which unit-tests
without the filesystem; the wrapper loads the document and writes to standard
output.

The renderer flattens the scene into a `Graph` keyed by `u32` id, built once from
the `VoxMain`. Working in `u32` keeps vxl from naming `branded_id::U32Id`, the id
type voxcore uses but does not re-export. The flattening reads each node's name,
child node ids, and child object ids, and tallies a placement count per node and
object. A node's placement count is its parent references plus a root listing; an
object's is its parent references. The counts drive both marks: two or more
placements is instanced, zero is unplaced.

Every line ends with one bundled tag: `name [Node <id>]` or `name [Object <id>]`,
with extra comma-separated fields appended. The kind and id always show, so the
id that correlates instances lives in the tag itself rather than a separate
column.

Instancing renders every placement expanded and appends `Instance` to the tag at
each one, as `name [Node <id>, Instance]`, the faithful view of a DAG that places
a shared node once per path. `--collapse-instances` expands only the first
placement and prints each later one as a `[Node <id>, Instance, Collapsed]` stub,
tracked by a set of already-expanded ids; the shared id ties a collapsed stub
back to its expanded occurrence. An object placed by several nodes takes the same
`Instance` field, but it is a leaf, so collapse does not apply to it. The flag is
a plain presence bool, matching the `--collapse-ancestors`/`--collapse-descendants`
family rather than the settable `--ext` style.

The tree keeps document order rather than sorting children by name the way the FBX
and Voxel Max views do, because a voxel-json document already orders its roots,
child nodes, and child objects, so preserving that order is both deterministic and
the truthful rendering. Child nodes print before child objects within a parent,
following the struct field order.

A traversal guard appends `Cycle` to the tag of a node found on its own ancestor
chain and does not re-enter it. The loaders build a `VoxMain` without validating,
so a document with a `childNodes` cycle would otherwise recurse forever.
Instancing across sibling branches is a diamond, not a cycle, so only an ancestor
repeat stops the walk.

The tree is split into a `Root` section of each root's subtree and an `Unplaced`
section, each under a bare header line printed only when its section is non-empty.
`Unplaced` lists nodes that are neither a root nor a child, then objects no node
places, each in listing order. Unplaced nodes render with their subtrees, which
surfaces child nodes reachable only through an unplaced parent and so absent from
the root tree. Orphan objects match the `--select` convention that an unreferenced
object has its bare name as its path, and `info` already reports every object.

`hierarchy show` renders only the markdown tree and takes no `--layout`, unlike
the other read reports with their `pretty-json` and `compact-json` forms. The
scene graph reads as a tree, so a JSON layout was dropped for this command, which
also keeps it off the shared `ReportLayout` and its `serde_json` path. A
machine-readable graph can return later if a caller needs one.

The `pattern` glob resolves against per-placement node paths. `enumerate_node_paths`
walks the roots then the unplaced nodes depth first, building each node's path as
the chain of node names from its section root, and records one entry per
placement, so an instanced node matched on one route shows only on that route. The
same branch set that guards the render guards the walk, so a cyclic document still
terminates. The paths go to the shared `match_glob` engine; the matched paths and
their proper prefixes become the filter's matched and ancestor sets. A pattern
that matches nothing is an error, exiting non-zero, matching the tyt hierarchy
command. `PathGlob` normalizes the pattern with a leading `**/`, the same type and
rule the `--select` glob uses, so a bare pattern matches at any depth.

Rendering threads an in-match flag and the current path. Above a match the walk
keeps only the child nodes whose path leads to a match and drops child objects,
which no node path names; in or below a match the whole subtree shows. Orphan
objects, which have no node path, drop entirely under a filter, though objects
still show under a matched node. Both the `Root` and `Unplaced` sections filter
this way, and an emptied section, header included, is omitted.

`--collapse-ancestors` prints the matches as a flat list rather than in their
sections, each behind an `[Ancestors]` marker, dropped when the match is a section
root with no chain to hide. `--collapse-descendants` replaces a matched node's
children with a `[Descendants]` marker. The markers are bracketed to match the
`[Node ...]` tag family and to read as placeholders rather than node names; the
earlier `(ANCESTORS)`/`(DESCENDANTS)` spelling followed the FBX view, before the
tag redesign. Both flags act only with a pattern. The recursion moved onto a
`Walk` struct so the growing option set does not bloat a parameter list: the
graph, the collapse flags, and the filter are fields, while the per-branch prefix,
path, in-match flag, and cycle set stay method arguments.

`branded-id` is a dev-dependency, used only by the tests to build hierarchy nodes
from returned ids and to fabricate a cyclic state for the cycle guard. The shipped
crate names no branded id; the render path stays on `u32`.
