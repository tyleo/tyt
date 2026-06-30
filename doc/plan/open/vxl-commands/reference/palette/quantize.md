# `vxl palette quantize`

*Part of [`vxl palette`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

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
[Palettes](../../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#palettes).
