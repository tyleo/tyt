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
5. `--fill-color <color>` (default `white`): a `#RRGGBBAA` hex or a name like
   `white`. Under the default `--fill-mode solid` every voxel takes this one
   color, written as the single cell of the document's one palette. Under
   `--fill-mode surface` each voxel instead samples its color from the source
   mesh's material at that surface point, so `--fill-color` does not apply there.

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
