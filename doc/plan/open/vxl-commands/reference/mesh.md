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
4. `--ambient-occlusion [true|false]` (default `false`): when on, bakes
   per-vertex ambient-occlusion darkening at concave junctions into vertex
   colors. Settable boolean: bare `--ambient-occlusion` means `true`.
5. `--select <glob>` / `--select-index <index>`: output only the matching
   objects. Both repeat, and the result is the union of all values. `--select`
   takes a name glob; `--select-index` takes an integer, `a-b` range, or comma
   list; see [Object selectors](conventions.md#object-selectors). Given neither,
   every object is output.

## Material and texture maps

Each unique merged material in the meshed geometry becomes one texel in a
compact atlas, and the mesh's UVs sample it. A material map is one image whose
channels are filled from the merged material's attributes, so every map shares
the same atlas and differs only in which attributes it reads. Attributes a cell
omits fall back to their spec defaults from
[Attributes](../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#attributes),
so a map never fails for a missing attribute.

Each preset takes an optional path that defaults to the mesh stem plus the map
name, matching the `{stem}-mse.png` style. Presets may be combined, and any
number of custom maps may be added with `--map`.

1. `--albedo [path]`: RGBA base color from `rgba`. Four channels.
2. `--orm [path]`: glTF occlusion-roughness-metallic packing, R = `occlusion`,
   G = `roughness`, B = `metallic`. Three channels.
3. `--metallic-roughness [path]`: glTF metallic-roughness packing, G =
   `roughness`, B = `metallic`, R = `0`. Three channels.
4. `--mse [path]`: the custom MSE packing, R = `metallic`,
   G = smoothness (`1 - roughness`), B = `emissive`. Three channels. This is
   the voxel-native form of the MSE texture the material tooling builds from
   image maps.
5. `--emissive [path]`: grayscale `emissive` strength. One channel.
6. `--occlusion [path]`: grayscale `occlusion`. One channel.
7. `--map <path>:<channels>`: a custom packing. `<channels>` is a
   comma-separated list of `R=<expr>`, `G=<expr>`, `B=<expr>`, and optional
   `A=<expr>`, where `<expr>` is an attribute name, `1-<attribute>` for an
   inverted attribute such as `1-roughness`, or the constant `0` or `1`. The
   channel count is the number of channels named; an omitted channel is `0`.
   Repeatable, once per output image. For example
   `--map model-mse.png:R=metallic,G=1-roughness,B=emissive` reproduces
   `--mse`, and swapping `G=roughness` writes roughness instead of smoothness.
