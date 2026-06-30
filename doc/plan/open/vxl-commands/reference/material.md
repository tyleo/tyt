# `vxl material`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl material <input> [output-stem] [maps] [options]
```

Bakes the material maps from [`vxl mesh`](mesh.md) without writing any geometry,
so you can produce or re-bake textures for a mesh you already have. It takes the
same map flags as `mesh`, `--texture <name> [path]` for the presets and
`--texture-map <path> <channels>` for a custom packing, plus the
`--computed-occlusion-*` options that tune a baked `computed-occlusion` map; see
[Material and texture maps](mesh.md#material-and-texture-maps). The default
`output-stem` is the input stem, and each preset path defaults to that stem plus
the map name.

`material` and `mesh` derive the atlas identically under the same `--atlas`
mode, so the maps `material` writes are byte-for-byte the maps a `mesh` run with
the same input, object selection, and atlas mode would produce, and they line up
with that mesh's UVs. With `--atlas palette` the maps depend only on the palette
and are shareable across every mesh on it; with `--atlas unwrap` they are
unwrapped per mesh. Either way you can iterate on materials without re-meshing.

1. `--from <format>`: source voxel format. Inferred from the input extension
   when omitted.
2. `--atlas` `palette` | `unwrap` (default `palette`): atlas layout, the same as
   `mesh`; see [Material and texture maps](mesh.md#material-and-texture-maps).
3. `--select <glob>`: restrict the material set to objects whose name matches
   the glob, the same selector as `mesh`. Repeatable; unions with
   `--select-index`. See [Object selectors](conventions.md#object-selectors).
4. `--select-index <index>`: restrict the material set to objects at the given
   position, an integer or an `a-b` range, the same selector as `mesh`.
   Repeatable; unions with `--select`. See
   [Object selectors](conventions.md#object-selectors). Given no selector of
   either kind, the default covers every object.

At least one map must be requested; with no `--texture` or `--texture-map` the
command reports the available presets and exits non-zero, since there is nothing
to bake.
