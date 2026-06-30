# `vxl voxelize`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl voxelize <input> [output] (--side-length <n> | --voxel-size <s>) [options]
```

Rasterizes a mesh into a voxel grid. This is the inverse of [`vxl mesh`](mesh.md).
The default output path is the input stem with the `.voxj` extension. The
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
[Coordinate System](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#coordinate-system).

`voxelize` writes a voxel-json document and accepts the same output options as
`to voxj`: `--format`, `--optimize`, `--position-encoding`,
`--sample-encoding`, `--ext`, and `--edit-state`. Those default the same way
they do there.
