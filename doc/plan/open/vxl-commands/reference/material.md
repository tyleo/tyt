# `vxl material`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl material <input> [output-stem] [maps] [options]
```

Bakes the material maps from [`vxl mesh`](mesh.md) without writing any geometry,
so you can produce or re-bake textures for a mesh you already have. It takes the
same map flags as `mesh`, `--texture <preset>` for the presets and bundles,
`--texture-name` / `--texture-name-prefix` for naming the preset images,
`--texture-map <file-name> <channels>` for a custom packing, and
`--define-property` for naming custom properties, plus the
`--computed-occlusion-*` options that tune a baked `computed-occlusion` map; see
[Material and texture maps](mesh.md#material-and-texture-maps). The default
`output-stem` is the input stem, and each preset image defaults to that stem plus
the preset name.

`material` and `mesh` derive the atlas identically under the same `--atlas`
mode, so the maps `material` writes are byte-for-byte the maps a `mesh` run with
the same input, object selection, atlas mode, and layer would produce, and they
line up with that mesh's UVs. With `--atlas palette` the maps depend only on the palette
and are shareable across every mesh on it; with `--atlas unwrap` they are
unwrapped per mesh. Either way you can iterate on materials without re-meshing.

1. `--from <format>`: source voxel format. Inferred from the input extension
   when omitted.
2. `--atlas` `palette` | `unwrap` (default `palette`): atlas layout, the same as
   `mesh`; see [Material and texture maps](mesh.md#material-and-texture-maps).
3. `--select <glob>`: restrict the material set to objects by hierarchy path,
   matched as `hierarchy show` matches node paths so a node path selects its
   subtree, the same selector as `mesh`. Repeatable; unions with `--select-index`
   and may cover several objects. See
   [Object selectors](conventions.md#object-selectors).
4. `--select-index <index>`: restrict the material set to objects at the given
   position, an integer or an `a-b` range, the same selector as `mesh`.
   Repeatable; unions with `--select`. See
   [Object selectors](conventions.md#object-selectors). Given no selector of
   either kind, the default covers every object.
5. `--layer <index>` (default `0`): the object layer whose materials this bake
   reads, a 0-based index into the object's layers in reference order, the same
   selector as `mesh`. The bake reads one selected layer rather than applying
   the format's layer-override resolution, defaulting to the first.

At least one map must be requested; with no `--texture` or `--texture-map` the
command reports the available presets and exits non-zero, since there is nothing
to bake.
