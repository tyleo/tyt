# `vxl material`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl material <input> [output-stem] [maps] [options]
```

Bakes the material maps from [`vxl mesh`](mesh.md) without writing any geometry,
so you can produce or re-bake textures for a mesh you already have. It takes the
same map flags as `mesh`: the `--albedo`, `--orm`, `--metallic-roughness`,
`--mse`, `--emissive`, and `--occlusion` presets, and the
`--map <path>:<channels>` escape hatch. The default `output-stem` is the input
stem, and each preset path defaults to that stem plus the map name.

`material` and `mesh` derive the atlas identically: one texel per unique merged
material across the selected objects, in the same canonical order. So the maps
`material` writes are byte-for-byte the maps a `mesh` run with the same input
and object selection would produce, and they line up with that mesh's UVs.
That lets you iterate on materials without re-meshing.

1. `--from <format>`: source voxel format. Inferred from the input extension
   when omitted.
2. `--select <glob>` / `--select-index <index>`: restrict the material set to
   the matching objects, the same selectors as `mesh`; see
   [Object selectors](conventions.md#object-selectors). Both repeat. The default
   covers every object.

At least one map must be requested; with no map flags the command reports the
available maps and exits non-zero, since there is nothing to bake.
