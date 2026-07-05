# Implementation decisions

_Part of the [Vxl Command-Line Reference](../README.md)._

Code-level decisions made while building these commands, recorded as they land.
Command-design rationale lives in [design notes](design-notes.md); this log is
for implementation choices a reviewer of the Rust would want explained.

> **Note.** The entries below predate the voxel-json format redesign and record
> what was decided at the time; they are a historical record and are not rewritten
> to the new model. The redesign later renamed the palette entry "cell" to
> "material", moved the recommended PBR attribute names to the glTF
> metallic-roughness vocabulary (`rgba` became `baseColorFactor`, and so on, with
> `emissive` split into `emissiveFactor` and `emissiveStrength`), replaced the
> cross-layer merge with non-merging `layerPaletteRefs` selected by `mesh`'s
> `--layer`, renamed `--max-palette-cells` to `--max-palette-materials` and
> `--show-palettes` to `--show-layers`, and added a `--color-format`
> (`hex` | `float`, default `float`) encoding option. Where an entry below names
> an older term, read it against that rename; the redesign's own code-level
> decisions are logged in
> [voxj-redesign](../../voxj-redesign/reference/implementation-decisions.md).

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

## palette list

`palette list` follows the `info` report template rather than the richer
`palette show` machinery, since it is a flat per-palette overview with no value
collections, selectors, or swatches. It loads the state and renders a pure
function over the `VoxMain`, the same load-render-print split the other read
reports use. The Markdown layout is one `markdown_table` with the columns the
spec shows, `index`, `attributes`, `cells`, and `used by`; the JSON layouts emit
one record per palette with the keys `index`, `attributes`, `cells`, and
`used_by`, the object indices in place of the Markdown names.

Its layout is a command-specific `PaletteListLayout` rather than the shared
`ReportLayout`, because only `palette list` adds a `hierarchy` value beside
`markdown` and the two JSON forms, the same reason `palette show` carries its own
layout enum. `hierarchy` is the default, since the tree reads as the natural
shape of a palette with its nested attributes and referencing objects; the table
and JSON are opt-in through `--layout`. The `hierarchy` layout draws the listing
as a tree in the `hierarchy show` idiom: a `palettes` header over one bare-index
branch per palette, its cell count a `cellCount: <n>` leaf and its `attributes`
and `objects` as subtrees. The box-drawing glyphs those two trees share, the four
connector and extension constants, moved out of `hierarchy_show` into a
`tree_glyphs` leaf module both draw from, so the connectors cannot drift apart.
The renderer collects a palette's enabled child branches into a `HierarchyChild`
list first, then walks it, so the last enabled branch takes the closing connector
whichever fields are on; an empty subtree prints `objects: []` the way
`hierarchy show` prints `palettes: []`.

Trailing positional filters reuse `SelectIndex`, the index-or-range selector the
object selectors already parse, since a palette-index filter is the same grammar,
`1`, `5-10`, repeatable and unioned. A palette lists when any filter contains its
index, and none given lists every palette. A filter set that matches no palette
is an error, matching how `hierarchy show` treats a pattern that selects nothing,
so a stray index is caught rather than silently listing nothing.

Which fields render is a `PaletteListFields` of three settable booleans,
`--show-attributes`, `--show-cells`, and `--show-objects`, each defaulting to
shown in the `--ext` style so a bare `palette list` prints them all and
`--show-* false` drops one. The index is always shown, as the palette's identity.
The command parses the three flags and bundles them into the struct the trait
carries, the same parse-then-bundle split `hierarchy show` uses for its views. A
dropped field leaves out its Markdown column, its JSON key, and its hierarchy
branch alike, so the three layouts stay consistent.

The `used by` column is computed by scanning the objects once per palette and
keeping those whose `iter_palette_refs` name it. An object appears once however
many times it references the palette, since the filter is a boolean any-match, so
an unvalidated document that references one palette twice still lists the object
once. Markdown shows the referencing object names and JSON their indices, the
same name-versus-id split `info` draws between its Markdown and JSON.

Building `list` surfaced four helpers already inlined in `info` and, for the JSON
tail, in `validate` too. They moved to shared `implementation` functions rather
than being duplicated a third time: `row` and `md_cell` join the `markdown_table`
module they feed, `attribute_names` and `to_json_string` become their own leaf
modules. `to_json_string` is the pretty-or-compact serialize plus trailing
newline that `info`, `validate`, and `list` share, so the read commands' JSON
stays byte-identical in form. `palette show` keeps its own serialize, since it
returns the string with an `expect` rather than propagating a `Result`, and
retyping it was not worth the churn.

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

## Per-texel sampling

`per-texel` and texture-aware `auto` add a base-color texel sampler on top of the
per-primitive path. `Mesh::is_textured` routes `auto`: any material with a
base-color texture samples per-texel, else per-primitive.

Texture images need no new dependency. The `gltf` crate's `import` feature, which
voxsmith already enables, pulls the `image` crate and decodes every referenced
PNG or JPEG, so `import_slice` returns the pixels the reader previously discarded.
voxsmith stays off `tyt-image` and the standalone `image` crate. A decoded glTF
image, whatever its channel count and 8 / 16 / float depth, normalizes to a
row-major RGBA8 `MeshTexture`; base color keeps its sRGB storage and decodes to
linear per sample. The reader also carries per-vertex texture coordinates (the
base-color texture's `TEXCOORD` set, `None` when absent) and one base-color
binding per material (image slot, linear factor, wrap modes), deduplicated by
glTF image index so shared textures decode once.

`GridSpace` is the shared world-to-grid map: the min corner and per-axis voxel
size. The rasterizer and the sampler both build it from the same bounds, so a
sampled surface point floors into the same cell the occupancy grid filled. This
replaced the rasterizer's inline size math, the one place the two mappings could
have drifted.

The rasterizer records, per surface cell, the first covering triangle's index
(not just its material), so the sampler can read that one triangle's texture, and
its material comes from `triangles[covering].material`. `sample_base_color`
supersamples each textured triangle rather than point-sampling each cell, so a
voxel spanning many texels averages them instead of aliasing onto one. Each
triangle scatters a barycentric lattice, its density tied to the longest
grid-space edge (about two samples per voxel, floored at two interior points and
capped at sixteen), each sample seated in a sub-triangle interior off the shared
vertices. A sample maps to its grid cell and interpolated coordinate, the texel
decodes to linear and multiplies the linear factor, and the running sum and count
accumulate. A sample lands only in the cells its own triangle covers (gated on
`grid.triangle[cell] == this triangle`), so a cell's color, its finish, and its
fallback all source from the one recorded covering triangle rather than mixing
whatever grazes it. The per-cell mean re-encodes to sRGB.

A surface cell the scatter's lattice grazes without a sample landing in it (a
sliver clipped at the cell's edge) would otherwise fall to the flat factor, which
on a white-`baseColorFactor` mesh painted stray white voxels through the model.
So a zero-sample surface cell point-samples its covering triangle at the cell
center (barycentric projection clamped onto the triangle), guaranteeing every
textured surface cell a texture color. The 8-bit re-encoding is the epsilon
merge: cells whose color rounds to the same stored row collapse to one palette
cell, keyed on the color bytes and the finish bit patterns.

The three color modes share one path. `resolve_materials` builds a per-cell
`MeshMaterial` list (`flat` paints one fill color; the sampling modes read each
surface cell's covering material, overriding its base color with the sampled
texel when per-texel), then a single `build_palette` dedups that list into cells
and emits the per-voxel samples. A `solid` interior takes the fill color or, via
a six-connected flood that carries each filled cell's nearest surface cell index,
that surface cell's resolved material.

The first per-texel commit sampled base color only; a follow-up (below) extends
the same scatter pass to the other four attributes. The tests embed a synthetic
textured glTF, encoding small PNGs with the `png` dev-dependency (already in the
lock through the gltf image decoder); a sampler unit test asserts every textured
surface cell resolves, locking in the coverage guarantee.

## Per-texel PBR maps

The per-texel sampler was extended from base color to the full material:
metallic, roughness, emissive, and occlusion each sample their own glTF texture on
the same scatter pass, so `sample_material` (renamed from `sample_base_color`)
returns a per-cell `MeshMaterial` rather than a base color and `resolve_materials`
takes it whole. An attribute whose material has no texture keeps the flat factor
`mesh` bakes, so the sampler's output is a superset of the per-primitive one.

The color space is decoded at the sample site, not stored on the texture, so one
image feeding maps of different kinds decodes correctly for each. `MeshTexture` is
a color-space-neutral RGBA8 store. Base color and emissive decode sRGB
(`to_linear_rgba`); metallic-roughness and occlusion are straight linear data
(`to_rgba`, byte over 255, no gamma), since decoding those through the sRGB curve
would corrupt them (a roughness `128/255 = 0.50` would read `~0.21`). The glTF
packings: metallic-roughness is blue times the metallic factor and green times the
roughness factor; occlusion is `1 + strength * (red - 1)`; emissive is the sRGB
texel times the emissive RGB factor, collapsed to its strongest channel, matching
the flat path. `KHR_materials_emissive_strength` is still not applied.

Each texture names its own TEXCOORD set (`info.tex_coord()`), so a triangle
carries one coordinate set per map slot in `MeshTriangleUvs` and the reader
resolves each slot to the set that map declares, reading each distinct set once.
The sampler reads its slot's coordinates and never touches set indices; a map
present without coordinates for its set, or absent, leaves that attribute at the
flat factor. The four bindings share a `MeshSampler` (image plus wrap modes) and
group per material in `MeshMaterialMaps`, replacing the lone base-color binding;
`Mesh::is_textured` now routes `auto` on any map, not only base color.

Each accumulated attribute divides by one shared per-cell sample count, since
every sample reads all of the covering material's present maps at once; a surface
cell the scatter grazes point-samples its covering triangle at the cell center,
covering all its maps together. New synthetic-glTF tests lock in the linear
metallic-roughness decode, the sRGB emissive collapse, the occlusion strength
formula, and each map reading its own TEXCOORD set.

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

All three methods are built, in all three spaces (oklab, lab, rgb). `median-cut`
recursively splits the widest color axis at its median; `octree` builds a
fixed-depth octree over the color cube and folds the least-populated all-leaf
nodes up until the leaf count fits, so it merges the rarest colors first; `kmeans`
seeds centroids by farthest-point (deterministic, no random start) and runs
population-weighted Lloyd iterations to a step cap. All cluster on the same
`Point` set and collapse each cluster to its most-sampled representative, so the
method only changes the grouping. Octree's eight-way folds are coarser than the
two-way median split, so it can stop a little under the cap. `--dither` is built
too (see [Dithering](#dithering)); every reduction control now applies when the
reduction fires and is inert when it does not. The cap fires quietly, never
failing: reduction is the designed default, so a note would print on nearly
every run.

## Dithering

`--dither` (`floyd-steinberg` | `ordered`, default `none`) is built for both
methods. It differs from the `--method` options in kind, not degree.
`median-cut` / `octree` / `kmeans` only change how cells cluster; the collapse
then repaints every voxel of a merged cell onto one representative uniformly,
through voxcore's `remove_cell`. Dithering is a per-voxel remap: each voxel
independently snaps to the nearest representative given the diffused error, so
voxels of one original color deliberately land on different representatives. The
material-follows-color rule still holds: a dithered voxel adopts the whole
representative row, so the pattern lands in the material, not just the color.

When `dither != none`, a per-object pass runs after the clusters and their
representatives are chosen but before the cell-level collapse:

1. Cluster and pick representatives exactly as the no-dither path does. The
   cluster coordinates are already in the working space, so each colored cell's
   color and each representative's snap coordinates are read straight off the
   clusters, once, and shared across objects.
2. For each object referencing the palette, walk its live voxels in voxcore's
   voxel-id raster order (`x*Y*Z + y*Z + z`, so `z` varies fastest) via
   `iter_live`, recovering each position with `voxel_position`.
3. For each voxel, take its original color's clustering-space coordinate, add the
   diffused error, find the nearest representative by Euclidean distance (ties to
   the lowest cell id), and, when that differs from the current cell, reassign the
   voxel with `retain_voxel`: read the voxel's full sample row and swap only this
   palette reference's cell. A voxel on a colorless survivor is skipped.
4. Diffuse the snapping error to not-yet-visited neighbors (floyd-steinberg only).

The collapse then runs unchanged: after the pass no live voxel samples a
non-representative colored cell, so `remove_cell`'s repaint is a no-op and only
the cell drop takes effect, then `gc` compacts. The reduced palette is the
representative cells plus the untouched survivors, and every representative
survives even if the dither left it unused, so the final cell count matches the
no-dither path.

Error diffusion, per method:

- `ordered` adds a per-axis offset read from a repeating 3D Bayer threshold
  matrix, scaled to the palette's spacing, before the nearest-representative snap.
  No error buffer, fully deterministic, position-only. The matrix is side `4` (64
  levels), one doubling of a 2x2x2 base that numbers the cube corners by parity
  (the even-parity corners before the odd), the 3D analog of the classic
  `[[0, 2], [3, 1]]` Bayer base: `M(p) = 8*base(p mod 2) + base(p/2 mod 2)`. Each
  axis reads the matrix at a rotation of the voxel position so the three channels
  decorrelate, and a raw threshold maps to `[-0.5, 0.5) * spacing`. `spacing` is
  the mean distance from each representative to its nearest other representative in
  the clustering space, so the offset perturbs a color by about one palette step;
  a lone representative has spacing `0`, disabling the offset.
- `floyd-steinberg` carries a per-voxel error, sparse (a `HashMap` keyed by voxel
  id, since only diffused voxels hold one), and pushes the snapping error forward
  to the not-yet-visited neighbors. 2D Floyd-Steinberg has a canonical kernel
  (`7/16`, `3/16`, `5/16`, `1/16`); 3D has none, so this defines one over the
  three raster-forward axis neighbors, weights summing to 1: `(x, y, z+1)` = `3/8`,
  `(x, y+1, z)` = `3/8`, `(x+1, y, z)` = `2/8`. Error pushed past a grid edge is
  dropped, as at a 2D image border.

Both diffuse in the clustering space (oklab by default), so the error is
perceptually meaningful. The traversal and error buffer are per object, since
each object is its own grid; the reduction still runs once over the shared
palette, then dithers each referencing object. The reassignment needs mutable
object access, so voxcore's `VoxMain` gained an `object_mut` accessor mirroring
`object`. Determinism holds with no RNG: object order, raster order, the
nearest-representative tie-break, the Bayer matrix, and the forward diffusion are
all fixed, so a given input always yields the same pattern.

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
the object subtrees. A later change added the `--show-palettes` subtree, then
another replaced the single `--show-bounds`/`--show-extents` pair with the six
edit/runtime geometry flags; see
[Edit- and runtime-grid geometry](#edit--and-runtime-grid-geometry).

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
transform lives on a node. A bare subtree header sits over its value lines,
`transform` over `position`/`rotation`/`scale`, each value naming its field and
bracketing its vector, as `position: [x, y, z]`, to the view's decimal precision.
It folds into the node's ordered children ahead of the real children so the
box-drawing connectors stay correct. The object geometry rows and the `palettes`
subtree fold in after the object line the same way.

A node's world transform folds the parent chain into one lossy world transform
rather than building and decomposing a matrix, matching how the `com.tyleo.game`
engine composes: rotation is the Hamilton product down the chain, scale the
component-wise product, and position the running
`parentT + parentR.rotate(parentS * childT)`. The scale is lossy because a
rotation between non-uniform scales introduces shear a per-axis scale cannot
hold, the same tradeoff the engine names in `GetLossyWorldScale`. A `world`
transform applies the fold whole; a `world` origin transforms a single grid
corner through it. The primitives, the quaternion product, component-wise
multiply, transform compose and transform-point, and a quaternion-to-euler, live
in `ty_math`.

A node's world transform depends on its route, since an instanced node has one per
placement, so the fold threads through the walk from the identity at each section
root, and each match also stores its parent's world transform during path
enumeration, so `--collapse-ancestors` still places a match in world space without
its hidden chain. Rotation renders as Tait-Bryan euler in `Rz*Ry*Rx` order so a
single-axis turn reads on its own component; the rotation samples confirm it, `40`
about z showing `0, 0, 40` and `30` about y then `40` about z showing `0, 30, 40`.

`--show-transforms` takes `[space] [rot-unit] [precision]`, the FBX arg shape,
parsed in the command into a `TransformView`. All the subtree views bundle into
one `HierarchyViews` value the trait carries, so the render entry point takes a
single argument rather than growing a parameter per flag; the earlier flat
signature had reached a `too_many_arguments` allow.

`--show-palettes` is a bare flag rather than a view struct, since a palette
reference carries no space or precision to tune. It appends a `palettes` subtree
under each object, beside the geometry rows, since palettes are referenced by an
object, not a node. Each child reads `index: {cells: <count>}`, one per reference
in `iter_palette_refs` order with no dedup, so the subtree mirrors the object's
real reference list rather than a set. The index is the referenced palette's
`to_u32`, which equals its position in `iter_palettes` for a freshly loaded state
that numbers ids `0..count`, the same index `palette show` prints, and the
palette resolves through `VoxMain::palette`; a reference the state does not hold
prints a `missing palette <id>` marker like the walk's other missing-id lines.
The count is the palette's `cell_count`, its size, not how many of its cells the
object's voxels sample. An object with no reference prints an empty `palettes: []`
leaf instead of a childless header, which also keeps `palettes` an unconditional
last child so the geometry rows above it stay correct.

`branded-id` is an `impl`-gated dependency. The render path names its `U32Id`
through the `NodeId` and `ObjectId` aliases, and the tests use it to build
hierarchy nodes from returned ids and to fabricate a cyclic state for the cycle
guard. It was dev-only while the render path stayed on `u32`; the `Scene` refactor
that dropped the `u32` projection moved it into the shipped crate.

### Edit- and runtime-grid geometry

The first cut of object geometry had one `--show-bounds`/`--show-extents` pair
with a `local`/`world` arg, where `world` reported the axis-aligned box after the
placing node's transform. That conflated two distinct questions: which grid to
measure and which space to measure it in. An object carries two grids. The
runtime grid is the tight box around its live voxels, read from
`VoxObject::live_extent`. The edit grid is the author's build volume,
`VoxObject::bounds` at `VoxObject::origin`, which a document with `editState`
records with margin around the runtime grid. So the pair became six flags,
`--show-{edit,runtime}-{origins,bounds,extents}`, the grid named in the flag.

Each row is measured relative to the placing node, the frame the user asked for:
bounds' `min` and `max` are the grid's corners offset from that node, extents is
`max - min`, and origin is the `min` corner alone. Origin is the one value a
world space is meaningful for, a specific corner as a scene point, so only the
origin flags keep the `[space]` arg; bounds and extents drop it and take a lone
`[precision]`, since a `world` axis-aligned box is a different quantity best left
out until asked for. The edit box always contains the runtime box, the invariant
the `voxj` spec states, so the world-box math the first cut carried, the
abs-rotation extents expansion, is gone; a `world` origin is a single
`transform_point`.

The edit grid can be absent: coincident with the runtime grid, it carries no
authoring margin. Rather than omit the row, which reads the same as the flag
being off, an absent edit value prints `null`, the document's own absent marker,
matching the JSON layouts of `palette show`. `edit_present` reuses the exact
margin test `info` applies for its edit-bounds column, so the two reports agree
on when an edit grid is distinct.

The runtime grid is never absent, so runtime rows never print `null`. An object
with no live voxels still has a runtime grid, a zero-size box; it prints one at
the object's origin, matching the `0x0x0` bounds and origin `info` reports for an
empty object. An earlier cut printed `null` here, but an empty object's grid is
zero, not missing, and the file records that zero. The one loss is the position:
for an empty object that also has an edit grid, `origin` is the edit origin,
since a `VoxObject` keeps a single origin and derives the runtime box from live
voxels, of which an empty object has none.

The rows render through an `ObjectRow` list built per object, a `Value` line or a
`Bounds` min/max subtree, each carrying its `Option` value so the `null` form is
one branch. Building the list first lets the object fold its enabled rows and the
`palettes` subtree into one ordered child sequence, so the last row and the last
subtree take the closing connector without the per-flag `is_last` juggling the
first cut used.

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

## mesh

The mesh engine lives in voxsmith, not vxl, mirroring `voxelize`: vxl gains no
mesh dependency of its own, and the `gltf` crate plus the meshing and glTF
writing sit behind voxsmith's `gltf` feature. voxsmith exposes
`object_to_mesh_geometry` (a `VoxObject` to a `MeshGeometry` of positions,
normals, and triangle indices) and two writers, `object_to_glb_bytes` and
`object_to_gltf_bytes`, sharing an internal `object_to_gltf_document`. The
`object_to_*` names keep `mesh` a noun throughout and leave a bare
`to_gltf_bytes(&VoxMain)` free for a future whole-scene exporter.

`MeshGeometry` carries its positions and normals as ty-math `TyVector3F32`, not
bare `[f32; 3]`, so the mesher reuses the crate's vector math rather than
open-coding it: the outward-winding test is `TyVector3::cross(...).dot(&normal)`,
and the writer's glTF accessor bounds are a `component_min_with` /
`component_max_with` fold. The vectors flatten to the `[f32; 3]` glTF wants with
`to_array` at the one buffer-packing boundary.

One slice sweep serves all three `--method` strategies, parameterized by two
booleans: `naive` emits every voxel face, `culled` only solid-empty boundary
faces, and `greedy` fuses coplanar boundary faces into maximal rectangles over
the slice mask. The mesher works in voxel-grid space, Z-up, winding each quad
counter-clockwise outward by a corner-cross test robust to either in-plane axis
orientation. Real-world scale and the axis flip are the writer's job, so the
mesher stays a pure grid-space function that unit-tests without any glTF.

The writer builds glTF with `serde_json::json!`, not the `gltf_json` structs:
voxsmith builds the `gltf` crate with `default-features = false`, which turns off
gltf-json's `names` and `extras` features and removes those struct fields, so
hand-constructing the typed structs would be brittle. `serde_json` gives a fixed
schema with no feature-gated fields; the `.glb` container is still assembled with
`gltf::binary::Glb`, and a `.gltf` embeds its one buffer as a base64 data URI so
the text form stays a single self-contained file. Vertices bake from Z-up grid
space to glTF Y-up by `(x, y, z) -> (x, z, -y)` then a uniform scale, the exact
inverse of the voxelizer's `(x, y, z) -> (x, -z, y)`; it is a proper rotation, so
outward winding is preserved. voxsmith gains `base64` and `serde_json` behind its
`gltf` feature.

`mesh` outputs one object, chosen by the `--select` / `--select-index` selectors,
and errors unless exactly one resolves. To keep flag knowledge out of the
implementation layer, the work splits across two `Dependencies` methods:
`resolve_objects` loads the document and resolves the selectors to matching
object indices (document order, deduplicated, with no flag-named errors), and
`mesh_object` meshes the object at a given index. The command orchestrates the
two and owns the mesh-specific "exactly one object" policy and its
`--select` / `--select-index` guidance. The cost is that the command loads the
document twice, once to resolve and once to mesh, since it cannot hold a
`voxcore` value across trait calls (voxcore is behind the `impl` feature); this
mirrors `voxelize`, which parses its glTF twice for the same reason, and the
accepted trade keeps `resolve_objects` reusable by `material`, `quantize`, and
`remap`.

The resolver returns plain `usize` object indices rather than voxcore ids, so the
command, which is not behind the `impl` feature, passes them back through
`mesh_object` without naming a voxcore type. Its core is a pure `select_objects`
over a `VoxMain` that unit-tests without the filesystem: it builds each object's
hierarchy paths from the roots (one path per placement in the DAG, guarding
against a cycle), then matches them through the shared `pathspec` gitignore
engine, the same one `hierarchy show` uses (`GitIgnoreRegex::from_spans_ignore_inert`
compiles `--select`'s raw glob strings and `pathspec::is_path_match` tests each
object path as a file leaf). Because `is_path_match` walks a path's ancestor
directories, a selected node carries down to its whole object subtree with no
separate expansion, and `!` negation with last-match-wins prunes it, so `--select`
inherits the full `hierarchy show` glob semantics for free. This retired the old
`PathGlob` / `match_glob` globset path. vxl gains `pathspec` and `branded-id`
behind its `impl` feature, the former for the matcher and the latter to name the
voxcore ids the resolver compares internally.

The vxl-side `MeshMethod` is a `ValueEnum` mapped to voxsmith's method in the
impl; `MeshFormat` gains an `extension()` for the defaulted output path, and the
output format defaults to `glb` when neither `--to` nor the output extension
picks one. The material, vertex, atlas, computed-occlusion, and storage flags in
[mesh.md](mesh.md) are unbuilt; the shipped command is the geometry-only subset.

## mesh textures

The `--atlas palette` texture path lives in voxsmith behind the `gltf` feature,
the same split as the geometry mesher: voxsmith bakes and wires, vxl lowers the
parsed flags. `object_to_material_glb` and `object_to_material_gltf` return a
`MeshFiles { mesh, sidecars }`, the mesh bytes plus the loose image files, and a
geometry-free `object_to_material_atlas` bakes just the images for the future
`material` command. voxsmith gains `png` (0.18.1) behind `gltf` to encode the
atlas images.

The palette atlas keys on **used combos**, not the spec's full product. A
voxel's material is the tuple of cells it samples across the object's palette
layers; `resolve_used_materials` collects the distinct tuples the object uses,
first seen in raster order, so the atlas holds one texel per material the mesh
actually references. This is a deliberate deviation from
[mesh.md](mesh.md#material-and-texture-maps), which lays out the product of the
layer sizes so the atlas is shareable byte-for-byte across meshes: the used-combo
atlas is compact and per-mesh, not shared. The test file's objects are two-layer
(a 255-cell `rgba` palette and an 8-cell material palette), so the product would
be 2040 texels where a solid single-material object needs one; used combos keep
it small (a 210-material sphere bakes a 15x14 image). The shareable product is
left for a later pass if a mesh needs to reuse another's atlas.

Greedy meshing had to become material-aware: the geometry MVP merged coplanar
faces regardless of material, but a merged quad samples one texel, so a quad may
span only one material. `mesh_slices` keys the slice merge on a per-voxel
material index and records it per vertex in a new `MeshGeometry::materials`;
`object_to_mesh_geometry` is that with a constant key and no tracking, so the
pure-geometry path is byte-identical. The document builder adds a `TEXCOORD_0`
set placing every vertex at its material's texel center in a near-square layout
(`atlas_dimensions` / `texel_center`), read with a nearest-neighbor sampler and
`CLAMP_TO_EDGE`, so each face samples exactly its texel. All four corners of a
quad share the UV, a degenerate UV triangle that is valid and correct under
nearest sampling.

Reading a material merges its layers, a later layer winning, and fills a missing
attribute from its spec default (`attribute_defaults`), so a map never fails on
an omitted attribute. A scalar and a color component both resolve to a `0..1`
fraction the packing scales to a byte, with inversion as `1 - fraction`, so
`R=metallic` and `R=rgba.a` inject the same way. `rgba` bakes straight to sRGB
bytes, correct for `baseColorTexture`; the scalar data maps are linear, correct
for the data slots; no color conversion is needed. The baker reads each
`#RRGGBBAA` cell through `ty_math::TySrgbaColor::from_hex`, the one hex parser the
palette-reduction path also shares, rather than a bespoke one. `computed-occlusion`
errors, since only an unwrap layout holds a per-face value.

The glTF slot each map fills is flag policy the command owns, mapped to
voxsmith's `MaterialSlot` in the impl the way `MeshMethod` is: `albedo` to
`baseColorTexture`, `metallic-roughness` to `metallicRoughnessTexture`, `orm` to
both `occlusionTexture` and `metallicRoughnessTexture` sharing one image,
`occlusion` to `occlusionTexture`, and `emissive` to `emissiveTexture` with the
emissive factor set to full. A preset with no standard slot (`mse`,
`metallic-smoothness`, the single-channel scalars) and every custom
`--texture-map` packing carry `MaterialSlot::None` and are listed under the
material's `extras.vxl.maps` by name, so a generic viewer ignores them and a
custom pipeline finds them.

The `emissive` preset is the one preset whose bake is not a literal channel
copy. Voxel Json models emissive as a strength scaling `rgba`, but glTF's
`emissiveTexture` is an RGB color, so a bare strength in one channel would glow
that channel's color (red) and replicating it across RGB would glow a flat
white. The `MaterialBake::EmissiveColor` bake instead scales the base color by
the strength in linear light, through the shared `ty_math` `TySrgbaColor` /
`TyLinearRgbaColorF64` round trip, so a surface glows in its own color and full
strength round-trips to the base color. The raw `emissive` attribute stays a
plain scalar channel for `--texture-map` and the `mse` packing; only the preset
lowers to the color bake, since only the preset targets the color slot.

`ResourceStorage` chooses where the images go, mapped to voxsmith's own enum in
the impl. Embedded packs a GLB image into the binary chunk as a buffer view and a
text-glTF image into a data URI; external writes a loose `.png` the mesh
references by relative name; both embeds and also writes the loose copy. The
default follows the target, `embedded` for `.glb` and `external` for `.gltf`, and
the impl writes each returned sidecar beside the output. The enum also backs the
deferred `--palette-storage`.

The flag arity is settled per option, since clap groups a repeatable multi-value
option only at fixed arity. `--texture-map <path> <channels>` is fixed at two
values chunked by two, the `palette show` pattern. `--texture <name> [path]` and
`--define-attribute <name> <key> [type]` have an optional trailing token, so each
occurrence is one whitespace-split value parsed by `FromStr` (matching
`AttributeBinding`'s form): `--texture albedo` is one token, and the path or type
override is a quoted `--texture "albedo out.png"`. `AttributeBinding` dropped its
palette index; a binding now reads the merged value across layers.

The command resolves the `--texture-map` channels against the
`--define-attribute` bindings and validates them: a binding alias becomes its
concrete key, a color attribute (a binding typed `color`, or the built-in `rgba`)
requires a component and a scalar rejects one, matching the mesh reference. The
`--texture` presets bypass the bindings and read the spec attributes, as the
reference promises. The resolved maps pass through `mesh_object` as flag-agnostic
`MeshTextureMap`s, so `implementation/` never sees a flag. `--atlas unwrap` and
any `computed-occlusion` map error as a later pass.
