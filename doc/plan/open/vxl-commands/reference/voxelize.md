# `vxl voxelize`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl voxelize <input> [output] (--side-length <n> | --scale <meters>) [options]
```

Rasterizes a mesh into a voxel grid. This is the inverse of [`vxl mesh`](mesh.md).
The input is a glTF mesh, text (`.gltf`) or binary (`.glb`); glTF is the only
mesh format read for now. The default output path is the input stem with the
`.voxj` extension. The grid resolution is set exactly one of two mutually
exclusive ways: a voxel count with `--side-length` or a real-world voxel size
with `--scale`.

1. `--from` `gltf` | `glb`: source mesh format, glTF text or binary. Inferred
   from the input extension when omitted.
2. `--side-length <n>`: grid resolution in voxels along the longest axis. The
   other axes are sized to preserve aspect, and the result is fit tight to
   `bounds`. Use this to cap detail at a known voxel count.
3. `--scale <meters>`: the edge length of one voxel in meters. Each axis count
   is the mesh extent on that axis in meters divided by `<meters>` and rounded
   up, so the same `<meters>` yields a consistent real-world voxel size across
   meshes of different sizes. Mutually exclusive with `--side-length`.
4. `--fill-mode` `solid` | `surface` (default `solid`): how the mesh fills the
   grid. `solid` rasterizes the surface and flood-fills the volume it encloses,
   producing a filled body, and expects a watertight mesh. `surface` rasterizes
   only the voxels the triangles pass through, leaving a hollow shell.
5. `--material-mode` `auto` | `per-primitive` | `per-texel` | `flat` (default
   `auto`): where each voxel's color and material come from. `--fill-mode` sets
   the geometry; this sets the color, the two are independent.
   1. `per-primitive` reads each mesh material's flat factors (base color,
      metallic, roughness, emissive, occlusion), giving one palette cell per
      material, so an untextured or stylized mesh stays exact with a tiny palette.
   2. `per-texel` samples those maps at each voxel's surface point, area-averaged
      over the voxel's footprint rather than point-sampled so fine texture does
      not alias into a muddy palette, capturing spatial detail at the cost of a
      larger palette.
   3. `flat` reads nothing from the mesh and paints the one `--fill-color`.
   4. `auto`, the default, picks `per-texel` when the mesh carries textures and
      `per-primitive` when it does not.

   Every mode writes the same attributes [`mesh`](mesh.md) bakes back, `rgba`,
   `metallic`, `roughness`, `emissive`, and `occlusion`, so a voxelized model
   round-trips through `mesh`.
6. `--fill-color` `none` | `<#RRGGBBAA>` (default `none`): the color of voxels
   that have no sampled surface. Its role depends on `--material-mode`:

   |                            | `--fill-color none`                           | `--fill-color #RRGGBBAA`             |
   | -------------------------- | --------------------------------------------- | ------------------------------------ |
   | `flat`                     | whole object white                            | whole object that color              |
   | `per-primitive`/`per-texel`| exterior sampled, interior its nearest surface | exterior sampled, interior that color |

   Only the interior voxels a `--fill-mode solid` body invents have no surface; a
   hollow `--fill-mode surface` shell is all surface, so under the sampling modes
   `--fill-color` does nothing there.
7. `--max-palette` `<n>` | `none` (default `256`): the most cells the document's
   palette may hold. Sampling can yield many distinct materials, `per-texel`
   especially; when the count exceeds `<n>` the palette is reduced to it and a
   note is written to standard error, never failing and never silently dropping
   cells. `256` keeps each per-voxel sample index within one byte (the format
   packs it at `ceil(log2(cells))` bits) and matches the familiar 256-color
   ceiling; `none` disables the cap for bit-exact materials. Reduction clusters
   on `rgba` and a merged cell takes its cluster representative's whole row, so
   material follows color: materials that land in one color cluster collapse to
   one real representative cell, not an averaged one. This is the same reduction
   [`palette quantize`](palette/quantize.md) runs, so `--max-palette <n>` matches
   piping the output through `palette quantize --count <n>`.
8. `--method`, `--space`, and `--dither`: the palette-reduction controls shared
   with [`palette quantize`](palette/quantize.md), defaulting the same way
   (`median-cut`, `oklab`, `none`). They shape the `--max-palette` reduction and
   are inert when it does not fire; `--dither` diffuses the snapping error across
   the voxels in 3D order.

The format carries no physical units: one unit is one voxel, and real-world
scale comes from hierarchy-node transforms. `--side-length` is a voxel count,
not an edge length. `--scale` reads the source mesh's real-world size only to
choose the grid counts; the written document is still unitless. glTF is
meter-native, and any scene- or node-level scale on the mesh is applied before
voxelizing, so two glTF exports of the same object at different authored scales
voxelize alike, mirroring [`vxl mesh`](mesh.md)'s `--scale`. When `--scale` is
used, `voxelize` records `<meters>` as the placing node's scale so the assembled
model keeps its source dimensions; `--side-length` has no real-world size to
record. See
[Coordinate System](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#coordinate-system).

`voxelize` writes a voxel-json document and shares `to voxj`'s encoding options:
`--format`, `--encoding-preset`, `--position-encoding`, and `--sample-encoding`,
which default the same way they do there. It does not take `--ext` or
`--edit-state`: a voxelized mesh has no source `ext` block to carry and no
editor build volume to record.
