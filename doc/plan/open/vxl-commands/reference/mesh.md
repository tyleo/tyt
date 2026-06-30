# `vxl mesh`

*Part of the [Vxl Command-Line Reference](../README.md).*

```
vxl mesh <input> [output] [options]
```

Triangulates voxels into a mesh and optionally bakes material textures the
mesh's UVs sample. The default output path is the input stem with the mesh
extension; the mesh format is inferred from the output extension or set with
`--to`.

By default `mesh` outputs every object as pure geometry: each object's voxel
grid is meshed on its own, with no hierarchy-node transform applied, since the
common case is pulling leaf objects out without placement. Pass `--select` or
`--select-index` to choose which objects to output; see
[Object selectors](conventions.md#object-selectors). Assembling a placed scene
from the hierarchy, baking the node transforms and instancing in
[Hierarchy Nodes](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#hierarchy-nodes),
is a separate mode left for a later pass.

1. `--to` `fbx` | `obj` | `gltf`: target mesh format. Inferred from the output
   extension when omitted.
2. `--from <format>`: source voxel format. Inferred from the input extension
   when omitted.
3. `--method` `greedy` | `culled` | `naive` (default `greedy`): meshing
   strategy. `greedy` merges coplanar, same-material faces into the fewest
   quads and has the lowest triangle count. `culled` emits one quad per
   solid-empty boundary face without merging. `naive` emits all six faces of
   every solid voxel, including hidden interior faces, and has the highest
   triangle count. Choose `culled` or `naive` only when you need stable
   per-voxel topology for further per-face editing.
4. `--ambient-occlusion [true|false]` (default `false`): when on, bakes computed
   ambient-occlusion darkening at concave junctions. With `--atlas palette` it
   goes into vertex colors, since the shared texture cannot hold per-vertex
   variation; with `--atlas unwrap` it is baked into the map. Settable boolean:
   bare `--ambient-occlusion` means `true`.
5. `--atlas` `palette` | `unwrap` (default `palette`): material-map atlas layout;
   see [Material and texture maps](#material-and-texture-maps).
6. `--select <glob>` / `--select-index <index>`: output only the matching
   objects. Both repeat, and the result is the union of all values. `--select`
   takes a name glob; `--select-index` takes an integer or `a-b` range; see
   [Object selectors](conventions.md#object-selectors). Given neither, every
   object is output.

## Material and texture maps

A material map is one image whose channels are filled from a material's
attributes, so every map shares one atlas and differs only in which attributes
it reads. Attributes a cell omits fall back to their spec defaults from
[Attributes](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#attributes),
so a map never fails for a missing attribute. Wherever an attribute is read,
`smoothness` names the derived `1 - roughness`, so `smoothness` and `1-roughness`
are interchangeable, as are `1-smoothness` and `roughness`. `--atlas` sets how
the atlas is laid out and how the mesh's UVs index it:

1. `palette` (default): one texel per palette entry, placed at the entry's
   palette index. The atlas depends only on the palette, not the geometry, so
   every mesh on a palette gets a byte-identical texture and identical UVs, and
   meshes that share a palette share its maps. Many faces sample one texel. This
   is the compact, shareable form.
2. `unwrap`: each face takes its own texel from a per-mesh UV unwrap, so the
   atlas is unique to one mesh and larger. Use it to bake spatially varying data
   that one texel per material cannot hold, such as per-vertex ambient occlusion
   written into the map rather than into vertex colors.

Maps come from two flags, each repeatable once per output image: `--texture` for
the named presets and `--texture-map` for a custom packing.

`--texture <name> [path]` writes a preset map, repeatable, as in
`--texture albedo --texture orm`. The optional `path` overrides the default of
the mesh stem plus the name, the `{stem}-mse.png` style. The names are:

1. `albedo`: RGBA base color from `rgba`. Four channels.
2. `orm`: glTF occlusion-roughness-metallic packing, R = `occlusion`,
   G = `roughness`, B = `metallic`. Three channels.
3. `metallic-roughness`: glTF metallic-roughness packing, G = `roughness`,
   B = `metallic`, R = `0`. Three channels.
4. `metallic-smoothness`: Unity metallic-smoothness packing, R = `metallic`,
   A = `smoothness`, G and B = `0`. Four channels.
5. `mse`: the custom MSE packing, R = `metallic`, G = `smoothness`,
   B = `emissive`. Three channels. This is the voxel-native form of the MSE
   texture the material tooling builds from image maps.
6. `emissive`: grayscale `emissive` strength. One channel.
7. `occlusion`: grayscale `occlusion`. One channel.
8. `roughness`: grayscale `roughness`. One channel.
9. `smoothness`: grayscale `smoothness`. One channel.

`--texture-map <path> <channels>` writes a custom packing, also repeatable. The
`channels` argument is a comma-separated list of `R=<expr>`, `G=<expr>`,
`B=<expr>`, and optional `A=<expr>`, where `<expr>` is an attribute name,
`1-<attribute>` for an inverted attribute, or the constant `0` or `1`. The
channel count is the number of channels named; an omitted channel is `0`. For
example `--texture-map model-mse.png R=metallic,G=smoothness,B=emissive`
reproduces `--texture mse`, and swapping `G=roughness` writes roughness instead
of smoothness.
