# `vxl palette quantize`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette quantize <input> [output] --count <n> [--index 0] [--attribute rgba] [options]
```

Reduces the selected attribute of a palette to at most `--count` distinct
values and rewrites the affected sample channel to match.

The input is either a full voxj/voxjz document or a bare palette JSON file, a
top-level `palettes` array of the same shape as a remap `--target` file. A
document carries the voxels that sample the palette, so every such voxel is
rewritten and `--dither` can diffuse the snapping error in 3D voxel order,
narrowed to chosen objects by `--select` and `--select-index`. A bare palette
has no voxels, so only its entries are reduced and dithering does not apply. The
output mirrors the input, a document in its own format or a bare palette JSON.

1. `--count <n>` (required): the maximum number of distinct attribute values to
   keep.
2. `--index <n>` (default `0`): which palette to quantize.
3. `--attribute <key>` (default `rgba`): which attribute to cluster on.
4. `--method` `median-cut` | `octree` | `kmeans` (default `median-cut`):
   clustering algorithm.
5. `--space` `oklab` | `lab` | `rgb` (default `oklab`): distance metric used
   when clustering. Applies to `rgba`.
6. `--dither` `none` | `floyd-steinberg` | `ordered` (default `none`): error
   diffusion when snapping values, walking each object's voxels in 3D order, not
   a 2D image. Needs a document; a bare palette has no voxels to walk.
7. `--select <glob>`: dither only objects selected by hierarchy path, a node
   path selecting its subtree, effective only with `--dither` set. Repeatable;
   unions with `--select-index`. See
   [Object selectors](../conventions.md#object-selectors).
8. `--select-index <index>`: dither only objects at the given position, an
   integer or an `a-b` range, effective only with `--dither` set. Repeatable;
   unions with `--select`. Given no selector of either kind, every object is
   dithered. See [Object selectors](../conventions.md#object-selectors).

A cell is a row across all of a palette's attributes, so quantizing one
attribute must not silently destroy the others. `quantize` clusters only the
selected attribute and merges two cells into one only when they agree on every
attribute after quantization. Cells that quantize to the same value of the
selected attribute but differ elsewhere stay distinct. So `--count` bounds the
distinct values of the selected attribute, while the total cell count may
remain higher. See
[Palettes](../../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#palettes).
