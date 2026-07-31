# Orphaned options

_A companion to the [value language](value-language.md) and the
[profile language](profile-language.md): the options and features the old
[`vxl mesh` reference](../vxl-commands/reference/mesh.md) still holds that
the new plan has yet to absorb. The map flags are missing deliberately,
since the two languages replaced them._

## Shipped options

The languages leave these untouched; a new reference must respell them.

### The command form

`vxl mesh <input> [output]`. The default output path is the input stem
with the mesh extension, and the format comes from `--to`, else the
output extension, else `.glb`.

### `--to` and `--from`

The mesh format (`gltf` | `glb`) and the source voxel format, each
inferred from its file extension when omitted.

### `--voxel-size`

The real-world edge length of one voxel in meters (default `1.0`), a
uniform scale on vertex positions.

### `--method`

`greedy` | `culled` | `naive` (default `greedy`): the meshing strategy.

### `--atlas` and the palette atlas

`palette` is the shipped layout: one texel per distinct flattened
material, every map sharing the layout, UVs at texel centers with a
nearest sampler and clamped wrapping, per-mesh. The new plan leans on the
effective palette without respelling
[the atlas itself](../vxl-commands/reference/mesh.md#the-palette-atlas).
`unwrap` is the deferred second value.

### `--texture-shape`

`line` | `fit` | `square` | `pot` | `<n>` (default `pot`): the atlas
canvas.

### `--select` and `--select-index`

The object selectors and the exactly-one-object policy, shared with the
other commands through
[Object selectors](../vxl-commands/reference/conventions.md#object-selectors).

## Deferred features

Designed against the retired flags, so each needs reworking rather than
carrying.

### Vertex attribute maps

The value carriers folded into the language as
[`--write-vertex`](value-language.md#vertex-attributes), superseding
`--vertex`, `--vertex-target`, and `--vertex-map`. Still orphaned: the
index carriers (`palette-index`, `palette-layers`), their `PaletteData`
tables, and `--palette-storage`, which carry indices rather than values
and need a home of their own.

### Computed occlusion and the unwrap atlas

Computed occlusion folded into the language: `computedOcclusion` is a
supplied per-face symbol arriving with `--atlas unwrap`, and expressions
supersede the three `--computed-occlusion-*` tuning flags; see
[Computed occlusion](value-language.md#computed-occlusion). Still
orphaned: the unwrap layout itself, the face packing, the texel-per-face
UV generation, and the second UV set that carries per-face maps beside
the palette atlas.

### Scene assembly

A separate mode baking hierarchy-node transforms and instancing into one
placed mesh, and with it the story for outputting more than one object.
