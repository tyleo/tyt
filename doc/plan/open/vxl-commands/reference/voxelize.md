# `vxl voxelize`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl voxelize <input> [output] [--resolution <axis> <n> | --voxel-size <meters>] [options]
```

Rasterizes a mesh into a voxel grid. This is the inverse of [`vxl mesh`](mesh.md).
The input is a glTF mesh, text (`.gltf`) or binary (`.glb`); glTF is the only
mesh format read for now. The default output path is the input stem with the
`.voxj` extension. The grid resolution is set one of two mutually
exclusive ways: a voxel count along a chosen axis with `--resolution` or a
real-world voxel size with `--voxel-size`. When neither is given it defaults to
`--voxel-size 1`, one voxel per meter.

1. `--from` `gltf` | `glb`: source mesh format, glTF text or binary. Inferred
   from the input extension when omitted.
2. `--resolution <axis> <n>`: pin one axis to a voxel count of `<n>` and size the
   other axes to preserve aspect, fit tight to `bounds`. `<axis>` selects which
   axis `<n>` counts along: `long` the longest extent, `short` the shortest, or
   `x` | `y` | `z` a specific axis. Use this to cap detail at a known voxel count.
3. `--voxel-size <meters>` (default `1`): the edge length of one voxel in meters.
   Each axis count is the mesh extent on that axis in meters divided by `<meters>`
   and rounded up, so the same `<meters>` yields a consistent real-world voxel size
   across meshes of different sizes. Mutually exclusive with `--resolution`, and
   used with `<meters>` of `1` when neither flag is given.
4. `--fill-mode` `solid` | `surface` (default `solid`): how the mesh fills the
   grid. `solid` rasterizes the surface and flood-fills the volume it encloses,
   producing a filled body, and expects a watertight mesh. `surface` rasterizes
   only the voxels the triangles pass through, leaving a hollow shell.
5. `--material-mode` `auto` | `per-primitive` | `per-texel` | `flat` (default
   `auto`): where each voxel's color and material come from. `--fill-mode` sets
   the geometry; this sets the color, the two are independent.
   1. `per-primitive` reads each mesh material's flat factors (`baseColorFactor`,
      `metallicFactor`, `roughnessFactor`, `emissiveFactor`, `emissiveStrength`,
      `occlusionStrength`), giving one material per mesh material, so an untextured
      or stylized mesh stays exact with a tiny palette.
   2. `per-texel` samples those maps at each voxel's surface point, area-averaged
      over the voxel's footprint rather than point-sampled so fine texture does
      not alias into a muddy palette, capturing spatial detail at the cost of a
      larger palette.
   3. `flat` reads nothing from the mesh and paints the one `--fill-color`.
   4. `auto`, the default, picks `per-texel` when the mesh carries textures and
      `per-primitive` when it does not.

   Every mode writes the same properties [`mesh`](mesh.md) bakes back,
   `baseColorFactor`, `metallicFactor`, `roughnessFactor`, `emissiveFactor`,
   `emissiveStrength`, and `occlusionStrength`, so a voxelized model round-trips
   through `mesh`.
6. `--fill-color <#RRGGBBAA>`: the color of voxels that have no sampled surface,
   omitted for the default. Its role depends on `--material-mode`:

   |                            | `--fill-color` omitted                        | `--fill-color #RRGGBBAA`             |
   | -------------------------- | --------------------------------------------- | ------------------------------------ |
   | `flat`                     | whole object white                            | whole object that color              |
   | `per-primitive`/`per-texel`| exterior sampled, interior its nearest surface | exterior sampled, interior that color |

   Only the interior voxels a `--fill-mode solid` body invents have no surface; a
   hollow `--fill-mode surface` shell is all surface, so under the sampling modes
   a set `--fill-color` is rejected there.
7. `--max-palette-materials` `<n>` | `none` (default `256`): the most materials
   the document's palette may hold. Sampling can yield many distinct materials,
   `per-texel` especially; when the count exceeds `<n>` the palette is reduced
   to it, never failing and never silently dropping materials. Reduction is the
   designed default, firing on nearly every run, so it stays quiet. `256` keeps
   each per-voxel sample index within one
   byte (the format packs it at `ceil(log2(materials))` bits) and matches the
   familiar 256-color ceiling; `none` disables the cap for bit-exact materials.
   Reduction clusters on `baseColorFactor` and a merged material takes its cluster
   representative's whole set of values, so material follows color: materials that
   land in one color cluster collapse to one real representative material, not an
   averaged one. This is the same reduction [`palette quantize`](palette/quantize.md)
   runs, so `--max-palette-materials <n>` matches piping the output through
   `palette quantize --max-palette-materials <n>`.
8. `--method`, `--space`, and `--dither`: the palette-reduction controls shared
   with [`palette quantize`](palette/quantize.md), defaulting the same way
   (`median-cut`, `oklab`, `none`). They shape the `--max-palette-materials`
   reduction and are inert when it does not fire; `--dither` diffuses the
   snapping error across the voxels in 3D order.
9. `--name <name>`: the voxelized object's name. Defaults to the mesh's own name,
   the first mesh-bearing node's (its own preferred over its mesh's), falling
   back to the input file stem when the glTF names neither.

The format carries no physical units: one unit is one voxel, and real-world
scale comes from hierarchy-node transforms. `--resolution` is a voxel
count, not an edge length. `--voxel-size` reads the source mesh's
real-world size only to choose the grid counts; the written document is still
unitless. glTF is meter-native, and any scene- or node-level scale on the mesh
is applied before voxelizing, so two glTF exports of the same object at different
authored scales voxelize alike, mirroring [`vxl mesh`](mesh.md)'s
`--voxel-size`. When `--voxel-size` is used, `voxelize` records
`<meters>` as the placing node's scale so the assembled model keeps its source
dimensions; `--resolution` has no real-world size to record. See
[Coordinate System](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#coordinate-system).

`voxelize` writes a voxel-json document and shares `to voxj`'s encoding options:
`--format`, `--color-format`, `--encoding-preset`, `--position-encoding`, and
`--sample-encoding`, which default the same way they do there. It does not take `--ext` or
`--edit-state`: a voxelized mesh has no source `ext` block to carry and no
editor build volume to record.
