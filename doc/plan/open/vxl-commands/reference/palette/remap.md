# `vxl palette remap`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette remap <input> [output] (--target <file> | --target-index <n>) [options]
```

Remaps each voxel to its nearest entry in a target palette and rewrites the
sample channel. Because samples are cell indices into a palette, the target
must name another palette, so a target is required: either an external palette
file with `--target`, or a palette already in the input with `--target-index`.

The input supplies the palette to remap and is either a full voxj/voxjz
document or a bare palette JSON file, a top-level `palettes` array of the same
shape as a `--target` file. A document carries the voxels that sample the
palette, so every such voxel is remapped and `--dither` can diffuse the snapping
error in 3D voxel order, narrowed to chosen objects by `--select` and
`--select-index`. A bare palette has no voxels, so its entries are remapped in
place and dithering does not apply. The output mirrors the input, a document in
its own format or a bare palette JSON.

1. `--target <file>`: a JSON file holding a voxel-json `palettes` array, the
   same shape as a document's `palettes`. The target is its palette at
   `--target-index`.
2. `--target-index <n>` (default `0`): which palette is the target. Indexes the
   `--target` file's array when `--target` is given, otherwise a palette in the
   input document itself. Without `--target` it must name a palette other than
   the `--index` one being remapped.
3. `--target-attribute <key>` (default `rgba`): the attribute compared when
   finding the nearest entry, in the target.
4. `--index <n>` (default `0`): which palette in the input to remap from.
5. `--attribute <key>` (default `rgba`): which attribute in the input to
   compare.
6. `--space` `oklab` | `lab` | `rgb` (default `oklab`): distance metric for the
   nearest-value search.
7. `--dither` `none` | `floyd-steinberg` | `ordered` (default `none`): error
   diffusion when remapping, walking each object's voxels in 3D order. Needs a
   document; a bare palette has no voxels to walk.
8. `--select <glob>`: dither only objects selected by hierarchy path, a node
   path selecting its subtree, effective only with `--dither` set. Repeatable;
   unions with `--select-index`. See
   [Object selectors](../conventions.md#object-selectors).
9. `--select-index <index>`: dither only objects at the given position, an
   integer or an `a-b` range, effective only with `--dither` set. Repeatable;
   unions with `--select`. Given no selector of either kind, every object is
   dithered. See [Object selectors](../conventions.md#object-selectors).

When several input cells land on the same target entry they merge into it, each
remapped voxel adopting the target entry's whole row, so material follows color
(the compared attribute), the same rule [`quantize`](quantize.md) and
[`voxelize`](../voxelize.md) follow.

## Example

A `--target` file is a JSON document whose top-level value is a voxel-json
`palettes` array, the same shape as a document's `palettes`. A single-palette
target file:

```json
[
  { "attributes": ["rgba"], "data": [["#1D2B53FF"], ["#7E2553FF"], ["#008751FF"]] }
]
```

Remap a model onto it:

```
vxl palette remap model.voxj out.voxj --target target.json
```

A target file may hold several palettes; `--target-index` picks one. Reusing
the palette array from the format spec:

```json
[
  { "attributes": ["rgba", "metallic"], "data": [["#FF0000FF", 1]] },
  { "attributes": ["rgba"], "data": [["#00FF00FF"], ["#0000FFFF"]] },
  { "attributes": ["metallic", "roughness"], "data": [[1, 0.2]] }
]
```

`--target-index 1` selects the second palette, the two-color `rgba` layer, as
the target.

The input may itself be a bare palette file, since it has the same shape as a
`--target` file, so a palette can sit on either side of a remap:

```
vxl palette remap palette.json out.json --target target.json
```

With no voxels there is nothing to diffuse, so `--dither` and the object
selectors are unavailable on a bare palette.
