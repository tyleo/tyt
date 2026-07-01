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

Superseded: `match_glob` and `PathGlob` were removed when the shared gitignore
engine landed; see
[Gitignore-style pattern matching](#gitignore-style-pattern-matching). `--select`
will match with `pathspec` when the object selectors are built.

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

voxsmith gets a new `gltf` Cargo feature (beside the existing
`goxl` / `mvox` / `qbcl` / `vmax` / `voxj` features) that gates the glTF reader,
plus a private `_mesh` marker the `gltf` feature enables that gates the
format-independent mesh type and the voxelizer, so a future mesh format enables
`_mesh` the same way. Unlike the codec features these are a mesh-to-`VoxMain`
front end, not a load/save converter, so they do not turn on `_codec`; `gltf` is
`dep:gltf` plus the `import`, `names`, and `utils` gltf features (which pull
`gltf::import_slice`, the node and mesh `name` accessors, and the buffer
`Reader`). vxl's `impl` feature enables it through `voxsmith/gltf`.

The pipeline pivots through a generic `Mesh`, a soup of material-tagged triangles
(each vertex a `TyVector3F64` on the Z-up axes) plus a deduplicated material
table, so voxsmith reads any mesh format into one type the voxelizer consumes.
`from_gltf_bytes` (in `convert/gltf/`) parses a glTF or GLB byte slice into a
`Mesh`; `Mesh::extent` returns its meter extent so vxl can size the grid; and
`voxelize_mesh` (in `convert/voxelize/`) takes the mesh, the resolved voxel
counts, the fill mode, the material mode, the fill color, the node scale, and the
object name, returning a `VoxMain` of one object placed by one root node. A
future mesh format adds only its own reader; extent and voxelization are shared.
The object name resolves override first (`--name`), then the mesh's own name (the
first mesh-bearing node's, its own preferred over its mesh's), then a
`fallback_name` the caller passes, which vxl fills with the input file stem. When
the caller resolved the grid from `--meters-per-voxel` it passes `<meters>` as
the node scale so the assembled model keeps its source size; `--voxel-grid-length`
passes `1`. vxl parses the glTF once, into the `Mesh`, then reads its extent and
voxelizes it, so the resolution policy stays in vxl with voxsmith taking plain
counts. `import_slice` auto-detects `.glb` versus `.gltf` from the bytes, so
`--from` is accepted for the documented interface but does not steer parsing; a
`.gltf` with external `.bin` buffers cannot resolve from bytes and errors.

Grid resolution is resolved in vxl's `implementation/voxelize.rs` before the
voxsmith call, into a single voxel-count triple: `--voxel-grid-length` caps the
longest axis and sizes the others to preserve aspect, while `--meters-per-voxel`
divides each meter extent by `<meters>` and rounds up. The mutual exclusion of
`--voxel-grid-length` and `--meters-per-voxel` is a clap `ArgGroup` with
`required = true`, so exactly one is present. The two `Option` flags live only in
the command struct clap fills; `execute` collapses them into a `GridResolution`
enum (`VoxelGridLength(u32)` | `MetersPerVoxel(f64)`) that the trait and impl
take, so the "exactly one" the group enforces at the CLI is a type invariant
past it, not a pair of `Option`s a resolver has to reconcile.
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

`--material-mode` chooses the color source, `--fill-mode` the geometry, and the
two are independent. Reading the mesh fails the conversion as a
`voxsmith::Error::Gltf`, a new variant beside the codec ones.

`Mesh::extent` sizes the grid straight from the parsed triangles, so there is no
separate points-only pass. The rasterizer records, in one pass, the first
covering triangle's material per surface cell alongside the occupancy grid
(`VoxelGrid`), so geometry and color share one raster; a `solid` flood fill then
invents interior cells with no material. The rasterizer and the `VoxelGrid`,
material table, and triangle types are all format-independent (they live under
`internal/mesh/`); only the glTF reader is glTF-specific.

Every mode writes the five attributes `mesh` bakes: `rgba`, `metallic`,
`roughness`, `emissive`, `occlusion`. `flat` and the interior fill cell use a
default finish (matte, non-metal, unoccluded). `per-primitive` reads each glTF
material's flat factors: glTF's linear base color is sRGB-encoded to match the
`rgba` attribute while its alpha, carrying no gamma, is scaled directly;
`metallic` and `roughness` are the raw factors; `emissive` collapses glTF's
emissive color to its strongest channel, since Voxel Json models emissive as one
strength scaling `rgba` (the `KHR_materials_emissive_strength` multiplier is not
applied yet); and `occlusion` defaults to `1`, as glTF carries occlusion only in
a texture, no flat factor.

`--fill-color` parses in vxl to `Option<[u8; 4]>` (a `FillColor` of `none` or a
`#RRGGBB`/`#RRGGBBAA` hex; the color names the MVP accepted are gone), `None`
being the `none` default. A `solid` body's interior takes that color through one
shared fill cell when given, else adopts its nearest surface material by a
six-connected multi-source flood from the surface cells. A hollow `surface`
shell has no interior, so `--fill-color` is inert there rather than rejected; the
old guard is gone.

Deferred to later commits: the texel sampler and texture-aware `auto` (both fall
back to `per-primitive` for now).

## Palette reduction

The `--max-palette-cells` cap and the shared `--method` / `--space` / `--dither`
controls reduce a palette to at most N cells. The flag is named
`--max-palette-cells` (not `--count` or `--max-palette`) and `palette quantize`
takes the same name, since both run the identical operation: clustering M cells
into N > M is a no-op, so a "count" is really a ceiling. Its value is a
`MaxPaletteCells` of `none` or a positive count; `voxelize` defaults it to 256,
`quantize` will require it. The `--method` / `--space` / `--dither` trio is a
flattened `PaletteReductionOptions` clap group, paired with the per-command cap
flag into a plain `PaletteReduction` the trait carries, so the group can be
shared while the caps differ (default vs required).

`reduce_palette` is the engine, a public voxsmith operation on the assembled
`VoxMain`: reduction is a general voxel operation, not vxl policy, so it lives
beside voxsmith's other `VoxMain` work rather than in the CLI. voxsmith defines
the plain `ReductionMethod` / `ColorSpace` / `Dither` enums; vxl keeps its
`--method` / `--space` / `--dither` clap `ValueEnum`s and maps to them, exactly as
`FillMode` / `MaterialMode` map to their voxsmith counterparts, while the
`PaletteReductionOptions` group and the `--max-palette-cells` cap stay in vxl.
voxsmith builds the full palette and vxl caps it by calling the engine after
voxsmith returns the state. The material-follows-color rule falls out of voxcore's
`remove_cell`, which repaints every voxel of a merged cell onto a real
representative cell (never an average) and drops the merged cell; a final `gc`
compacts the holes. The representative is the most-sampled cell in its cluster
(ties to the lowest id), so the common color wins and the choice is deterministic.
Clustering is on the `rgba` color converted to the chosen space (a cell without
`rgba` survives untouched); alpha is not a clustering dimension but rides along in
the representative's row. The `branded-id` dependency moved to voxsmith with the
engine, so vxl no longer depends on it.

Only median-cut is built, in all three spaces (oklab, lab, rgb). `octree`,
`kmeans`, and any `--dither` but `none` error as not-yet-implemented, but only
when the reduction actually fires: under the cap the controls are inert, matching
the spec, so an unbuilt choice on a small palette is silent. The cap fires with a
note to standard error, never failing.

## Typed colors and the generic mesh

Two changes landed together, both moving voxsmith toward general, ty-math-founded
types.

Typed colors replaced the ad-hoc color math. `reduce_palette` used to convert
`[u8; 4]` to `[f64; 3]` through bare `linear` / `oklab` / `lab` functions, and the
glTF material reader carried its own sRGB encode; both now go through one typed
color family in `ty-math`, so the compiler forbids mixing spaces (no clustering an
sRGB value against a linear one) and the two duplicated sRGB implementations
collapse to one. The types live in `ty-math` beside its existing space-agnostic
`TyRgbaColor`, split by storage form versus compute form. `TySrgbaColor` is
byte-backed (`r` / `g` / `b` / `a: u8`), the 8-bit sRGB storage code voxj keeps as
`#RRGGBBAA`; it has no arithmetic, so the only way to compute is to leave for
linear via `to_linear_rgb`. `TyLinearRgbColor` / `TyOklabColor` / `TyCielabColor`
are the float working spaces, generic over `T` with f32/f64 aliases; the
conversions are impl'd at f64, where every consumer here lives and where the
high-precision perceptual matrices avoid the `excessive_precision` clippy lint an
f32 instantiation would trip (an OKLab-in-`u8` is meaningless, so the generic
ranges only over floats regardless). sRGB is the one space with a real 8-bit
story and never appears fractional here, so it stays byte-backed while linear and
the perceptual spaces stay float; 8-bit lives only at the storage boundary, which
is also how a GPU treats it (sample `RGBA8` sRGB, linearize, compute in float).
The C# types the user cited (`scratch/com.tyleo.game/Tyleo.Game/`: `TyRgbaColor`,
`TyHsvaColor`) model the value-struct convention, but have no perceptual spaces
and no linear/sRGB split, which was the gap to close.

The glTF-specific mesh became a general `Mesh`. The voxelizer already worked on a
material-tagged triangle soup rather than on glTF, so the reader was the only
glTF-bound piece; naming that seam gives a format-independent `Mesh` (triangles of
`TyVector3F64` in Z-up world space plus a material table) that any reader loads
into and that `voxelize_mesh` and `Mesh::extent` consume, a mesh-side echo of the
crate's format-to-`VoxMain` hub. It also drops the old double-parse: vxl parses
the glTF once into a `Mesh`, not once for the extent and again to rasterize. The
rasterizer keeps its `[f64; 3]` grid-space separating-axis math untouched,
converting from the mesh's `TyVector3F64` points at the one boundary; retyping the
tested SAT math would have added churn and risk for no payoff, since grid space is
a private implementation detail.

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

The markdown table leads with a `#` column of the 0-based cell index of each row,
prepended in `render_markdown` rather than in the shared `markdown_table`, so the
`info` tables stay unchanged. A row maps to a palette cell, so the index reads as
the cell number a selector or the other layouts refer to; only `markdown` gains
the column, since `row` and `column` already carry their headers and JSON records
the values positionally.

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
`pattern` glob with `--collapse-ancestors`/`--collapse-descendants`; the third
the `--show-transforms`/`--show-bounds`/`--show-extents` subtrees, local and
world.

`Dependencies::hierarchy_show` carries the load, render, and print together like
the other read reports, so the command struct only parses flags. Its core is a
pure `render` over a `VoxMain` that returns the output string, which unit-tests
without the filesystem; the wrapper loads the document and writes to standard
output.

The renderer works against the loaded `VoxMain` directly through a thin `Scene`
view. `VoxMain` stays the single source of truth for names, children, transforms,
and grid boxes, read back by id through its accessors as the walk needs them. The
only datum `Scene` derives is the placement count per node and object, tallied
once from the roots and every node's child lists. A node's placement count is its
parent references plus a root listing; an object's is its parent references. The
counts drive both marks: two or more placements is instanced, zero is unplaced.

The two count columns are `IdVec`s indexed by branded id rather than hash maps.
The render path never mutates the loaded state, and a freshly loaded document
numbers its ids `0..count` with no holes, so each column is sized once to the
live count and filled in place, which fits the counts better than growing a map
key by key. The tally ignores an id outside that range, so an unvalidated
document's dangling reference stays a no-op rather than an out-of-bounds panic,
and the missing node or object still renders through the walk's own missing-id
markers.

An earlier cut flattened the whole scene into a `Graph` of `u32`-keyed maps that
duplicated `VoxMain`, chosen so vxl would not name `branded_id::U32Id`, the id
type voxcore uses but does not re-export. Dropping it removed that copy and cut
the file's rendering code by about sixty lines, and it also ended the id erasure
that let a node id and an object id be mixed. `Scene` indexes its count columns
and threads its walk on branded ids through the aliases `NodeId` and `ObjectId`,
so the two id kinds are distinct types again. Naming `U32Id` makes `branded-id` a
real `impl`-gated dependency rather than a dev-only one. Its `Display` prints the
brand name, so the id in each tag formats through `to_u32` for the bare integer.

Every node and object line reads `name: {node: <id>}` or `name: {object: <id>}`,
the name as a prefix over a map of the entity's fields. The kind and id always
show, so the id that correlates instances lives in the tag itself rather than a
separate column.

The delimiters follow the data shape, like JSON: `{...}` wraps a map of
`key: value` fields, so an entity tag reads `{node: <id>, instance: <k>}`, and
`[...]` wraps an array, so a vector value reads `position: [x, y, z]`. Everything
else is a bare lowercase label: the `root` and `unplaced` section headers, the
`transform` and `bounds` subtree headers, and the `ancestors` and `descendants`
markers. Earlier spellings bracketed every annotation uniformly and capitalized
the labels, before this split the delimiters by shape and lowercased throughout.

A shared node, placed two or more times, adds an `instance: <k>` field counting
the placements already shown, so its first occurrence is `instance: 0` and each
repeat climbs from there, tracked by a per-id counter rather than a set. By
default every placement expands, the faithful view of a DAG that places a shared
node once per path. `--collapse-instances` expands only the first occurrence and
stops each repeat as a stub; no separate `collapsed` field is needed, since a
nonzero `instance` already says the occurrence is a repeat. An object placed by
several nodes takes the same `instance` field, but it is a leaf, so collapse does
not apply to it. The flag is a plain presence bool, matching the
`--collapse-ancestors`/`--collapse-descendants` family rather than the settable
`--ext` style.

The tree keeps document order rather than sorting children by name the way the FBX
and Voxel Max views do, because a voxel-json document already orders its roots,
child nodes, and child objects, so preserving that order is both deterministic and
the truthful rendering. Child nodes print before child objects within a parent,
following the struct field order.

A traversal guard adds a `cycle: true` field to a node found on its own ancestor
chain and does not re-enter it. The loaders build a `VoxMain` without validating,
so a document with a `childNodes` cycle would otherwise recurse forever.
Instancing across sibling branches is a diamond, not a cycle, so only an ancestor
repeat stops the walk.

The tree is split into a `root` section of each root's subtree and an `unplaced`
section, each under a bare header line printed only when its section is non-empty.
`unplaced` lists nodes that are neither a root nor a child, then objects no node
places, each in listing order. Unplaced nodes render with their subtrees, which
surfaces child nodes reachable only through an unplaced parent and so absent from
the root tree. Orphan objects match the `--select` convention that an unreferenced
object has its bare name as its path, and `info` already reports every object.

`hierarchy show` renders only the markdown tree and takes no `--layout`, unlike
the other read reports with their `pretty-json` and `compact-json` forms. The
scene graph reads as a tree, so a JSON layout was dropped for this command, which
also keeps it off the shared `ReportLayout` and its `serde_json` path. A
machine-readable graph can return later if a caller needs one.

The patterns resolve against per-placement paths, revised from the single glob
when the gitignore engine landed. `enumerate_placements` walks the roots then the
unplaced nodes depth first, building each node's path as the chain of names from
its section root and each object's path as its node's path plus the object name,
and records one entry per placement, so an instanced node matched on one route
shows only on that route; the orphan objects follow with their bare names. The
same branch set that guards the render guards the walk, so a cyclic document still
terminates. Each placement, tagged node or object, goes to
`pathspec::is_path_match` with its directory-ness; the selected paths and their
proper prefixes become the filter's `selected` and `visible` sets. A pattern set
that selects nothing is an error, exiting non-zero, matching the tyt hierarchy
command.

Rendering is driven by set membership rather than an in-match flag, so a git
exclude inside a selected subtree hides only that branch. A node shows when its
path is in `visible`, selected or leading to a selection; a child object shows when
its path is in `selected`. Both the `root` and `unplaced` sections filter this way,
and an emptied section, header included, is omitted. Because selecting a node
selects its whole subtree, a matched node's objects are selected too and render
beneath it.

`--collapse-ancestors` and `--collapse-descendants` act on match roots, the
selected placements whose parent is not itself selected, the entry point of each
selected subtree. `--collapse-ancestors` prints the match roots as a flat list,
each behind an `ancestors` marker, dropped when the root is a top-level node with
no chain to hide; a root that is an object prints as a leaf, a root that is a node
prints its subtree. `--collapse-descendants` replaces a match root's children with
a `descendants` marker. The markers print bare, like the section headers, so they
read as placeholders rather than names; an earlier spelling bracketed them, before
brackets were reserved for collections. Because the two collapse flags act only
with a pattern, they are bundled with the globs in a `PatternView`, so a collapse
flag cannot be set without a pattern; the resolved filter carries them, and the
command enforces the same shape at the CLI with a clap `requires`. The recursion
runs on a `Walk` struct so the growing option set does not bloat a parameter list:
the scene, the instance-collapse flag, the subtree views, and the filter are
fields, while the per-branch prefix, path, and cycle set stay method arguments.

`--show-transforms` prepends a `transform` subtree under each node, since a
transform lives on a node; `--show-bounds` and `--show-extents` append `bounds`
and `extents` under each object, since a grid box lives on an object. A bare
subtree header sits over its value lines, `transform` over `position`/`rotation`/
`scale`, `bounds` over `min`/`max`, and `extents` as one line, each value naming
its field and bracketing its vector, as `min: [x, y, z]`. Each vector prints to
the view's decimal precision, uniform across a local integer grid and a
world-space float. They fold into the node's ordered children so the
box-drawing connectors stay correct, the transform ahead of the real children and
the bounds and extents after the object line.

World space folds the parent chain into one lossy world transform rather than
building and decomposing a matrix, matching how the `com.tyleo.game` engine
composes: rotation is the Hamilton product down the chain, scale the component-wise
product, and position the running `parentT + parentR.rotate(parentS * childT)`.
The scale is lossy because a rotation between non-uniform scales introduces shear a
per-axis scale cannot hold, the same tradeoff the engine names in
`GetLossyWorldScale`. World bounds apply that transform to the object's grid box
with the abs-rotation extents trick, so the reported box is the axis-aligned bound
after placement. The primitives this needs, the quaternion product,
component-wise multiply, `rotate_extents_abs`, transform compose and
transform-point, and a quaternion-to-euler, were added to `ty_math` in the prior
commit.

A node's world transform depends on its route, since an instanced node has one per
placement, so the fold threads through the walk from the identity at each section
root, and each match also stores its parent's world transform during path
enumeration, so `--collapse-ancestors` still places a match in world space without
its hidden chain. Rotation renders as Tait-Bryan euler in `Rz*Ry*Rx` order so a
single-axis turn reads on its own component; the rotation samples confirm it, `40`
about z showing `0, 0, 40` and `30` about y then `40` about z showing `0, 30, 40`.

`--show-transforms` takes `[space] [rot-unit] [precision]` and the bounds flags
`[space] [precision]`, the FBX arg shape, parsed in the command into the
`TransformView` and `BoundsView` data structs the trait carries; the growing
signature takes a `too_many_arguments` allow, the house style beside `voxelize`.

`branded-id` is an `impl`-gated dependency. The render path names its `U32Id`
through the `NodeId` and `ObjectId` aliases, and the tests use it to build
hierarchy nodes from returned ids and to fabricate a cyclic state for the cycle
guard. It was dev-only while the render path stayed on `u32`; the `Scene` refactor
that dropped the `u32` projection moved it into the shipped crate.

## Gitignore-style pattern matching

`hierarchy show` takes several patterns matched with `.gitignore` rules,
replacing the single `match_glob`/`PathGlob` glob. The matcher is a new
dependency-light crate, `pathspec`, a Rust port of the C#
`com.tyleo.gitignore` package the game uses to enable loggers by hierarchical
name. It keeps that package's shape one-to-one: an `UnsignedGitIgnoreRegex` (a
compiled pattern plus its directory-or-file kind), a signed `GitIgnoreRegex` (the
unsigned pattern plus a sign a leading `!` flips), the `GitIgnoreRegexKind` enum,
and the `is_directory_match`/`is_file_match`/`is_path_match` aggregators, each
with an unsigned any-match variant. Pattern compilation delegates to `globset`
rather than the C# hand-built regex, so `*`, `?`, `[...]`, and `**` follow real
gitignore and the C# restrictive character class and literal `?` are dropped as
bugs. A pattern with no `/` and no `**` gets a `**/` prefix to float at any depth;
a leading or interior `/` anchors it. The array-and-span overload pairs collapse
to one slice function each.

It lives in its own crate rather than in vxl because it is reusable across tyt
tools and must stay free of `tyt-common` and `tyt-injection`, the same
independence rule vxl keeps; it sits in `projects/utilities` beside `ty-math` and
depends only on `globset`. `match_glob`, `PathGlob`, and the
`Dependencies::match_glob` method were removed with the switch, and vxl's direct
`globset` dependency gave way to `pathspec`.

Two deliberate refinements over the C# reference. `is_path_match` gained an
`is_dir` argument: the C# had none and folded the whole leaf path into the
directory walk, so a directory-only pattern like `*/` could select a leaf object
by its own name. Threading directory-ness matches the leaf as a directory or a
file by kind, so a trailing-slash pattern selects nodes but never an object by
name, while an object still rides in as part of a selected node's subtree. And the
constructors are fallible, since `globset` rejects a malformed glob, which
surfaces as a clear invalid-pattern error at the CLI.

The parent-directory rule is git-faithful, the choice the design notes left open:
`is_path_match` walks the ancestor directories shallowest first and the first
excluded ancestor prunes the whole subtree, so an excluded node cannot have a
descendant re-selected. The tree layer selects on the union of node and object
placements, then shows a node when it is selected or leads to a selection and an
object when it is selected; the collapse features key on the match roots, the
selected placements whose parent is unselected.
