# `vxl palette quantize`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl palette quantize <input> [output] --max-palette-materials <n> [--index 0] [--property baseColorFactor] [options]
```

Reduces the selected property of a palette to at most `--max-palette-materials`
distinct values and rewrites the affected sample channel to match.

The input is either a full voxj/voxjz document or a bare palette JSON file, a
top-level `palettes` array of the same shape as a remap `--target` file. A
document carries the voxels that sample the palette, so every such voxel is
rewritten and `--dither` can diffuse the snapping error in 3D voxel order,
narrowed to chosen objects by `--select` and `--select-index`. A bare palette
has no voxels, so only its entries are reduced and dithering does not apply. The
output mirrors the input, a document in its own format or a bare palette JSON.

1. `--max-palette-materials <n>` (required): the maximum number of materials to
   keep. The selected property is clustered to this many values and each cluster
   collapses to one material.
2. `--index <n>` (default `0`): which palette to quantize.
3. `--property <key>` (default `baseColorFactor`): which property to cluster on.
4. `--method` `median-cut` | `octree` | `kmeans` (default `median-cut`):
   clustering algorithm.
5. `--space` `oklab` | `lab` | `srgb` (default `oklab`): distance metric used
   when clustering. Applies to `baseColorFactor`.
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

A material spans all of a palette's array properties, one value per property.
Material
follows color: `quantize` clusters the selected property to at most
`--max-palette-materials` values, then collapses each cluster to one material
whose whole set of values is its representative's, so the other properties follow
the clustered one: materials in the same color cluster merge into one and any
appearance difference between them is lost to the representative.
`--max-palette-materials` therefore bounds the palette's material count, not just
the selected property's distinct values. The representative is an actual
material, never an averaged one, so every kept material is a real one. This is the
reduction [`voxelize`](../voxelize.md)'s `--max-palette-materials` applies inline,
sharing this command's `--method`, `--space`, and `--dither`. See
[Palettes](../../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#palettes).
