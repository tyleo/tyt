# `vxl palette remap`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette remap <input> [output] (--target <file> | --target-index <n>) [options]
```

Remaps each voxel to its nearest entry in a target palette and rewrites the
sample channel. Because samples are cell indices into a palette, the target
must name another palette, so a target is required: either an external palette
file with `--target`, or a palette already in the input with `--target-index`.

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
   diffusion when remapping, in 3D voxel order.

Remap merges input cells that land on the same target entry only when they
agree on every non-compared attribute, the same rule [`quantize`](quantize.md)
follows for multi-attribute cells.

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
