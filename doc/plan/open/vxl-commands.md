# Vxl Command-Line Reference

`vxl` is a command-line tool for working with voxel data. It converts between
voxel formats, meshes voxels into editable geometry, voxelizes meshes, bakes
material textures, and inspects and validates voxel-json documents.

This reference targets the voxel-json format. Its on-disk shape, encodings,
palette model, hierarchy, and validation rules are defined in the
[voxel-json file format spec](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md).
Sections below link into that spec rather than restating it, and any rule here
must agree with it.

A voxel-json file comes in two interchangeable forms with identical content:
`.voxj` (plain UTF-8 JSON) and `.voxjz` (a zip archive holding one `.voxj`
member). Every command that reads a voxel file accepts either form, recognized
by leading bytes (`{` versus `PK`) rather than by extension, as the spec
requires in [File Extensions](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#file-extensions).
The reference writes `.voxj` for brevity.

A document holds an ordered `palettes` array. Each palette declares an ordered
set of `attributes` (`rgba`, `metallic`, `roughness`, and so on) and lists its
cells as rows of values, one value per attribute. A voxel samples one cell per
palette its object references, and its material is the ordered merge of those
cells, with later palettes overriding earlier ones on shared attributes. This
model is defined in [Palettes](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#palettes).
Palette commands address a target with two shared options, `--index` (which
palette, default `0`) and `--attribute` (which attribute key, default `rgba`).

> Notation: `<required>`, `[optional]`, `[optional=default]`, and `flag` for a
> presence or settable boolean.

## Command overview

```
vxl to <format> <input> [output] [options]     convert between voxel formats
vxl mesh <input> [output] [options]            voxel -> editable mesh + material maps
vxl material <input> [output-stem] [options]   bake material maps only
vxl voxelize <input> [output] [options]        mesh -> voxel grid
vxl palette list <input> [options]             list a document's palettes
vxl palette show <input> [options]             print one palette
vxl palette quantize <input> [output] ...      reduce a palette's colors
vxl palette remap <input> [output] ...         remap voxels onto a target palette
vxl hierarchy show <input> [pattern] [options] print the scene graph
vxl validate <input> [options]                 check a document against the spec
vxl info <input> [options]                     report a document's contents
```

`vxl to` already ships. The rest are the subject of this plan.

## `vxl to <format>`

```
vxl to <format> <input> [output] [options]
```

Converts a voxel file from any supported input format to `<format>`, one of
`goxl`, `mvox`, `qbcl`, `vmax`, or `voxj`. This command exists today and is the
canonical place encodings and containers are chosen, so it is also how a
document is re-encoded, packed, and unpacked. See
[Re-encoding, packing, and unpacking](#re-encoding-packing-and-unpacking).

`to voxj` writes a voxel-json document and owns the encoding choice through
`--optimize`, `--position-encoding`, and `--sample-encoding`, and the output
container through `--format json|zip|pretty`. Those options map onto the spec's
[Voxel Encoding](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#voxel-encoding)
and [Choosing an Encoding](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#choosing-an-encoding).

## `vxl mesh`

```
vxl mesh <input> [output] [options]
```

Triangulates voxels into a mesh and optionally bakes material textures the
mesh's UVs sample. The default output path is the input stem with the mesh
extension; the mesh format is inferred from the output extension or set with
`--to`.

By default `mesh` outputs every object as pure geometry: each object's voxel
grid is meshed on its own, with no hierarchy-node transform applied, since the
common case is pulling leaf objects out without placement. Pass `--select` to
choose which objects to output; see [Object selectors](#object-selectors).
Assembling a placed scene from the hierarchy, baking the node transforms and
instancing in
[Hierarchy Nodes](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#hierarchy-nodes),
is a separate mode left for a later pass.

1. `--to` `fbx` | `obj` | `gltf`: target mesh format. Inferred from the output
   extension when omitted.
2. `--from <format>`: source voxel format. Inferred from the input extension
   when omitted.
3. `--method` `greedy` | `culled` | `naive` (default `greedy`): meshing
   strategy. `greedy` merges coplanar, same-material faces into the fewest
   quads and has the lowest triangle count. `culled` emits one quad per
   solid-empty boundary face without merging. `naive` emits all six faces of
   every solid voxel, including hidden interior faces, and has the highest
   triangle count. Choose `culled` or `naive` only when you need stable
   per-voxel topology for further per-face editing.
4. `--ambient-occlusion [true|false]` (default `false`): when on, bakes
   per-vertex ambient-occlusion darkening at concave junctions into vertex
   colors. Settable boolean: bare `--ambient-occlusion` means `true`.
5. `--select <selector>`: output only the matching objects. Repeatable; the
   result is the union of all selectors. A selector is an object index or a name
   glob; see [Object selectors](#object-selectors). Omitted, every object is
   output.

### Material and texture maps

Each unique merged material in the meshed geometry becomes one texel in a
compact atlas, and the mesh's UVs sample it. A material map is one image whose
channels are filled from the merged material's attributes, so every map shares
the same atlas and differs only in which attributes it reads. Attributes a cell
omits fall back to their spec defaults from
[Attributes](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#attributes),
so a map never fails for a missing attribute.

Each preset takes an optional path that defaults to the mesh stem plus the map
name, matching the `{stem}-mse.png` style. Presets may be combined, and any
number of custom maps may be added with `--map`.

1. `--albedo [path]`: RGBA base color from `rgba`. Four channels.
2. `--orm [path]`: glTF occlusion-roughness-metallic packing, R = `occlusion`,
   G = `roughness`, B = `metallic`. Three channels.
3. `--metallic-roughness [path]`: glTF metallic-roughness packing, G =
   `roughness`, B = `metallic`, R = `0`. Three channels.
4. `--mse [path]`: the custom MSE packing, R = `metallic`,
   G = smoothness (`1 - roughness`), B = `emissive`. Three channels. This is
   the voxel-native form of the MSE texture the material tooling builds from
   image maps.
5. `--emissive [path]`: grayscale `emissive` strength. One channel.
6. `--occlusion [path]`: grayscale `occlusion`. One channel.
7. `--map <path>:<channels>`: a custom packing. `<channels>` is a
   comma-separated list of `R=<expr>`, `G=<expr>`, `B=<expr>`, and optional
   `A=<expr>`, where `<expr>` is an attribute name, `1-<attribute>` for an
   inverted attribute such as `1-roughness`, or the constant `0` or `1`. The
   channel count is the number of channels named; an omitted channel is `0`.
   Repeatable, once per output image. For example
   `--map model-mse.png:R=metallic,G=1-roughness,B=emissive` reproduces
   `--mse`, and swapping `G=roughness` writes roughness instead of smoothness.

## `vxl material`

```
vxl material <input> [output-stem] [maps] [options]
```

Bakes the material maps from `vxl mesh` without writing any geometry, so you
can produce or re-bake textures for a mesh you already have. It takes the same
map flags as `mesh`: the `--albedo`, `--orm`, `--metallic-roughness`, `--mse`,
`--emissive`, and `--occlusion` presets, and the `--map <path>:<channels>`
escape hatch. The default `output-stem` is the input stem, and each preset path
defaults to that stem plus the map name.

`material` and `mesh` derive the atlas identically: one texel per unique merged
material across the selected objects, in the same canonical order. So the maps
`material` writes are byte-for-byte the maps a `mesh` run with the same input
and object selection would produce, and they line up with that mesh's UVs.
That lets you iterate on materials without re-meshing.

1. `--from <format>`: source voxel format. Inferred from the input extension
   when omitted.
2. `--select <selector>`: restrict the material set to the matching objects,
   the same selector as `mesh`; see [Object selectors](#object-selectors).
   Repeatable. The default covers every object.

At least one map must be requested; with no map flags the command reports the
available maps and exits non-zero, since there is nothing to bake.

## `vxl voxelize`

```
vxl voxelize <input> [output] (--side-length <n> | --voxel-size <s>) [options]
```

Rasterizes a mesh into a voxel grid. This is the inverse of `vxl mesh`. The
default output path is the input stem with the `.voxj` extension. The
resolution is set one of two mutually exclusive ways, exactly one required:

1. `--from` `fbx` | `obj` | `gltf`: source mesh format. Inferred from the input
   extension when omitted.
2. `--side-length <n>`: grid resolution in voxels along the longest axis. The
   other axes are sized to preserve aspect, and the result is fit tight to
   `bounds`. Use this to cap detail at a known voxel count.
3. `--voxel-size <s>`: the edge length of one voxel in the source mesh's units.
   Each axis count is the mesh extent on that axis divided by `<s>` and rounded
   up, so the same `<s>` yields a consistent real-world voxel scale across
   meshes of different sizes.

The format carries no physical units: one unit is one voxel, and real-world
scale comes from hierarchy-node transforms. `--side-length` is a voxel count,
not an edge length. `--voxel-size` reads the source mesh's units only to choose
the grid counts; the written document is still unitless. Because scale lives in
node transforms, `voxelize` can record `<s>` as the placing node's scale so the
assembled model keeps its source dimensions. See
[Coordinate System](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#coordinate-system).

`voxelize` writes a voxel-json document and accepts the same output options as
`to voxj`: `--format`, `--optimize`, `--position-encoding`,
`--sample-encoding`, `--ext`, and `--edit-state`. Those default the same way
they do there.

## `vxl palette list`

```
vxl palette list <input> [options]
```

Gives a one-line-per-palette overview of the whole document so you can see what
is there before printing any colors. Each row shows the palette index, its
ordered attribute keys, its cell count, and which objects reference it, which is
exactly the index and attribute that `palette show`, `quantize`, and `remap`
ask for. Example:

| index | attributes          | cells | used by            |
| ----- | ------------------- | ----- | ------------------ |
| 0     | rgba, metallic      | 12    | Object A, Object B |
| 1     | rgba                | 2     | Object B           |
| 2     | metallic, roughness | 1     | Object B           |

From there, `vxl palette show <input> --index 1` prints palette 1's colors.

1. `--json`: emit the listing as JSON, including per-palette attribute keys,
   cell count, and referencing object indices.

## `vxl palette show`

```
vxl palette show <input> [--index 0] [--attribute rgba] [options]
```

Prints one palette's selected attribute.

1. `--index <n>` (default `0`): which palette to show.
2. `--attribute <key>` (default `rgba`): which attribute to show.
3. `--format` `auto` | `swatch` | `string` (default `auto`): `auto` prints
   colored swatches for `rgba` and numeric values for every other attribute,
   since swatches are meaningful only for color. `swatch` forces colored
   swatches. `string` prints raw values, one per line: the `#RRGGBBAA` hex for
   `rgba` and the literal value otherwise, the form meant for piping into other
   tools.
4. `--json`: emit the palette as JSON instead.

## `vxl palette quantize`

```
vxl palette quantize <input> [output] --count <n> [--index 0] [--attribute rgba] [options]
```

Reduces the selected attribute of a palette to at most `--count` distinct
values and rewrites the affected sample channel to match. The default output
path is the input stem with `.voxj`.

1. `--count <n>` (required): the maximum number of distinct attribute values to
   keep.
2. `--index <n>` (default `0`): which palette to quantize.
3. `--attribute <key>` (default `rgba`): which attribute to cluster on.
4. `--method` `median-cut` | `octree` | `kmeans` (default `median-cut`):
   clustering algorithm.
5. `--space` `oklab` | `lab` | `rgb` (default `oklab`): distance metric used
   when clustering. Applies to `rgba`.
6. `--dither` `none` | `floyd-steinberg` | `ordered` (default `none`): error
   diffusion when snapping values. Dithering runs in the object's 3D voxel
   order, not a 2D image.

A cell is a row across all of a palette's attributes, so quantizing one
attribute must not silently destroy the others. `quantize` clusters only the
selected attribute and merges two cells into one only when they agree on every
attribute after quantization. Cells that quantize to the same value of the
selected attribute but differ elsewhere stay distinct. So `--count` bounds the
distinct values of the selected attribute, while the total cell count may
remain higher. See
[Palettes](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#palettes).

## `vxl palette remap`

```
vxl palette remap <input> [output] (--target <file> | --target-index <n>) [options]
```

Remaps each voxel to its nearest entry in a target palette and rewrites the
sample channel. Because samples are cell indices into a palette, the target
must name another palette, so exactly one target selector is required.

1. `--target <file>`: a voxel-json file whose palette is the target.
2. `--target-index <n>`: a palette already in the input file, by index.
3. `--target-attribute <key>` (default `rgba`): the attribute compared when
   finding the nearest entry, in the target.
4. `--index <n>` (default `0`): which palette in the input to remap from.
5. `--attribute <key>` (default `rgba`): which attribute in the input to
   compare.
6. `--space` `oklab` | `lab` | `rgb` (default `oklab`): distance metric for the
   nearest-value search.
7. `--dither` `none` | `floyd-steinberg` | `ordered` (default `none`): error
   diffusion when remapping, in 3D voxel order.

Remap merges input cells that land on the same target entry only when they
agree on every non-compared attribute, the same rule `quantize` follows for
multi-attribute cells.

## `vxl hierarchy show`

```
vxl hierarchy show <input> [pattern] [options]
```

Prints the scene graph as a tree with box-drawing glyphs, modeled on the FBX
hierarchy view. The graph is a DAG, not a tree: a node may have multiple
parents, which is instancing, and the roots are exactly the nodes listed in
`rootHierarchyNodes`. `show` marks shared and instanced nodes and lists
unplaced library nodes, defined as nodes that are neither a root nor a child,
so the structure stays visible rather than implying a strict tree. Each node
shows its name and its referenced child objects. See
[Hierarchy Nodes](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#hierarchy-nodes).

1. `pattern`: an optional glob matched against node paths. When set, only
   matched nodes and their ancestors print, or only matched nodes with
   `--collapse-ancestors`. `**/` is auto-prepended when the pattern does not
   already start with it.
2. `--show-transforms [space] [rot-unit] [precision]`: prepend each node's
   transform as a nested subtree. `space` is `local` (default) or `world`;
   `rot-unit` is `rad` (default) or `deg`; `precision` is the decimal precision
   for alignment (default `2`).
3. `--show-bounds [space] [precision]`: append each object's grid `bounds`
   subtree. In `world` space the bounds are reported as the axis-aligned box
   after the placing node's transform.
4. `--show-extents [space] [precision]`: append an extents subtree
   (`max - min`), with the same arguments as `--show-bounds`.
5. `--collapse-ancestors` (flag): with a `pattern`, hide the ancestor chain
   above each match and replace it with an `(ANCESTORS)` marker, omitted when
   the match is a root. No effect without a `pattern`.
6. `--collapse-descendants` (flag): with a `pattern`, hide the descendants of
   each match and replace them with a `(DESCENDANTS)` marker, omitted when the
   match has no descendants. No effect without a `pattern`.
7. `--json`: emit the graph as JSON, including root, instanced, and unplaced
   flags.

## `vxl validate`

```
vxl validate <input> [options]
```

Checks a voxel-json document against the spec's
[Validation](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#validation)
checklist and exits non-zero on any failure. The checks include a recognized
`version`; all indices in range; well-formed position data with correct decoded
byte lengths and zero pad bits; unique voxel positions; tight `bounds`; sample
arity matching `paletteRefs` and correct per-channel lengths; well-formed
palette rows and `rgba` strings; an acyclic hierarchy; no zero `scale`
component; unit `rotation` quaternions within tolerance; and, when present, an
`editState` whose edit grid contains each runtime grid. The one item a
validator cannot confirm, that sample order matches the position block's voxel
order, is reported as unverifiable.

1. `--json`: emit a structured report of every check and its result instead of
   human-readable output.

## `vxl info`

```
vxl info <input> [options]
```

Reports what a document contains, surfacing the format internals that voxel
counts and bounds alone miss: the `version`; per-object `bounds`, voxel count,
and the position and sample encodings in use; each palette's attribute set and
cell count; whether `editState` and `ext` namespaces are present; and the root,
instanced, and unplaced nodes in the hierarchy.

1. `--json`: emit the report as JSON.

## Re-encoding, packing, and unpacking

These are not separate commands. The `to voxj` command already chooses
encodings and containers, so it covers all three:

1. Re-encode or optimize: `vxl to voxj in.voxj out.voxj --optimize size`
   rebuilds every object with the smallest encoding pairing. Re-encoding
   positions reorders voxels, and `to voxj` regenerates the sample channels to
   match, which is the invariant from
   [Voxel Order](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#voxel-order).
   Pin one block with `--position-encoding` or `--sample-encoding` to search
   only the other.
2. Pack to the shipping form: `vxl to voxj in.voxj out.voxjz --format zip`.
3. Unpack to plain JSON: `vxl to voxj in.voxjz out.voxj`, optionally with
   `--format pretty` for readable output.

## Conventions and cross-command options

These hold across the commands above and match the existing `to` commands.

1. Input format is recognized by leading bytes or inferred from the extension,
   and overridden with `--from`. Mesh I/O format is inferred from the mesh
   extension or set with `--to` and `--from`.
2. Output paths are optional and default to the input stem with the new
   extension, so a defaulted `to voxj` writes `.voxj` and `--format zip` writes
   `.voxjz`.
3. Settable booleans follow the `--ext` style: a bare flag means `true`, an
   explicit `--flag false` turns it off, and the option has a default.
4. Palette addressing is one pair of options everywhere it appears, `--index`
   (default `0`) and `--attribute` (default `rgba`), rather than positional
   arguments, so optional values never trail required ones.
5. `--json` is available on the read-only reports: `palette list`,
   `palette show`, `hierarchy show`, `validate`, and `info`.

### Object selectors

`mesh` and `material` take `--select`, repeatable, to choose which objects to
output. Each matched object is meshed as pure geometry, with no hierarchy-node
transform. A selector matches objects one of two ways:

1. By index: an object index into the document's `objects`, written as a plain
   integer, a range `a-b`, or a comma-separated list, as in `0`, `2-5`, or
   `0,3,7`. Index is the canonical object reference in the spec.
2. By name: a glob over object names, where `*`, `?`, and `[...]` match as in a
   shell. Object names are flat, not paths, and are not guaranteed unique, so a
   glob may match several objects.

A selector is read as an index when it contains only digits, ranges, and
commas, and as a name glob otherwise. Multiple `--select` values union their
matches.

Selecting hierarchy nodes instead, to bake a node's subtree and transforms into
one larger placed mesh, is a separate mode left for a later pass.

## Design notes

Rationale for the non-obvious choices, for reviewers.

1. No standalone `optimize`, `pack`, or `unpack`. Every one is a special case
   of `to voxj`, which already owns encoding and container selection, so adding
   them would duplicate that logic and split the invariant that re-encoding
   positions must regenerate samples.
2. `mesh` plus `voxelize` rather than per-format `mesh fbx`. Inferring the mesh
   format from the extension with `--to` and `--from` matches how `to voxj` and
   `to vmax` infer source format, keeps one home for the material options, and
   avoids a subcommand per format. `voxelize` is the conventional verb for the
   inverse.
3. Material maps live on `mesh` as presets plus a `--map` escape hatch. The
   presets name the common packings, ORM and MSE included, so the common cases
   are one flag, while `--map` expresses any custom channel-to-attribute
   packing without a code change. This replaces the original single `--texture`
   flag that packed a filename, a channel count, a palette index, and a
   variadic cell list into one argument. The same map flags are exposed as a
   standalone `material` command so textures can be re-baked without re-meshing;
   both derive the same atlas, so the maps stay aligned to the mesh UVs.
4. `mesh` outputs objects as pure geometry, narrowed with `--select`. The main
   use is pulling leaf objects out with no transform data, so selection targets
   objects, by index, the canonical reference, or by name glob, rather than
   hierarchy nodes. One repeatable `--select` replaces the separate `--object`
   and `--object-name` flags. It is a flag, not a positional, because the
   optional `output` positional is the house convention and two trailing
   optional positionals would be ambiguous. Assembling a placed scene from
   hierarchy nodes, baking transforms and instancing, is a deferred, separate
   mode.
5. Quantize and remap state their multi-attribute rule. A cell spans every
   attribute, so reducing one attribute has to define what happens to cells
   that share that value but differ elsewhere. Both keep such cells distinct,
   bounding the selected attribute without silently dropping PBR distinctions.

## Future and nice-to-haves

1. A scene-assembly mode for `mesh` and `material`: selecting hierarchy nodes
   and baking their transforms and instancing into one larger placed mesh,
   complementing the pure-geometry object selectors.
2. stdin and stdout via `-`, so commands compose in pipelines.
3. A dry-run or preview mode for the destructive palette operations.
4. Additional mesh export targets beyond `fbx`, `obj`, and `gltf` as needed.
