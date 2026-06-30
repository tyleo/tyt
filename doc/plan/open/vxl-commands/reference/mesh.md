# `vxl mesh`

_Part of the [Vxl Command-Line Reference](../README.md)._

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

1. `--to` `fbx`: target mesh format. Inferred from the output
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
4. `--vertex-computed-occlusion [true|false]` (default `false`): bakes
   occlusion computed from the voxel geometry into the mesh's vertex colors,
   darkening concave junctions. Each face vertex takes its occlusion from the
   three voxels meeting at that corner, giving four discrete levels. The value
   lives on vertices, so it forces per-face resolution: with `--method greedy`
   the mesher merges two faces only when their occlusion matches along the
   shared edge, so flat runs still merge into large quads while concave seams
   split; `culled` and `naive` already carry one value per face corner. When a
   quad's four corners are uneven, the triangle diagonal is chosen to keep the
   darker pair together so interpolation does not seam. To write the same
   occlusion into a texture instead of vertex colors, use `computed-occlusion`
   under [Material and texture maps](#material-and-texture-maps). Settable
   boolean: bare `--vertex-computed-occlusion` means `true`.
5. `--computed-occlusion-strength <0..1>` (default `1.0`): scales how much
   computed occlusion darkens, from `0` for none to `1` for the full effect.
   Applies to both `--vertex-computed-occlusion` and the `computed-occlusion`
   map.
6. `--computed-occlusion-min-brightness <0..1>` (default `0.0`): floor on the
   brightness the deepest occlusion reaches, so crevices never darken below it.
   `0` lets occlusion reach black.
7. `--computed-occlusion-color-space` `linear` | `srgb` (default `linear`): the
   space the occlusion values are written in. `linear` is correct for glTF and
   other PBR data textures; `srgb` matches pipelines that multiply occlusion in
   sRGB space.
8. `--atlas` `palette` | `unwrap` (default `palette`): material-map atlas layout;
   see [Material and texture maps](#material-and-texture-maps).
9. `--select <glob>`: output only objects whose name matches the glob.
   Repeatable; the result is the union of every `--select` and `--select-index`
   value. See [Object selectors](conventions.md#object-selectors). Given no
   selector of either kind, every object is output.
10. `--select-index <index>`: output only objects at the given position, an
   integer or an `a-b` range. Repeatable; unions with `--select` as above. See
   [Object selectors](conventions.md#object-selectors).

## Material and texture maps

A material map is one image whose channels are filled from a material's
attributes, so maps that read attributes share one atlas and differ only in
which attributes they read. Attributes a cell omits fall back to their spec defaults from
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
   atlas is unique to one mesh and larger. Use it for spatially varying data
   that one texel per material cannot hold, such as `computed-occlusion`
   baked from the voxel geometry.

`computed-occlusion` is occlusion computed from the voxel geometry, the same
quantity `--vertex-computed-occlusion` writes to vertex colors. It
varies across a surface, so it cannot occupy a palette texel and always bakes
into an unwrap layout for its own image. Under `--atlas palette` the shared
palette maps keep the mesh's primary UV set and the occlusion image takes a
second, unwrap UV set, so one mesh carries both the compact shared material
maps and a per-mesh occlusion bake. Under `--atlas unwrap` every map already
shares the one unwrap set and occlusion is another image on it. The second UV
set needs a mesh format that stores more than one: `gltf` and `fbx` do, `obj`
does not, so an `obj` target cannot pair palette material maps with
`computed-occlusion`; use `--atlas unwrap` or output `gltf` or `fbx` instead.

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
8. `computed-occlusion`: grayscale occlusion computed from the voxel geometry
   rather than read from the `occlusion` attribute. One channel. Always bakes
   into an unwrap layout; see the atlas notes above.
9. `roughness`: grayscale `roughness`. One channel.
10. `smoothness`: grayscale `smoothness`. One channel.

`--texture-map <path> <channels>` writes a custom packing, also repeatable. The
`channels` argument is a comma-separated list of `R=<expr>`, `G=<expr>`,
`B=<expr>`, and optional `A=<expr>`, where `<expr>` is an attribute name,
`1-<attribute>` for an inverted attribute, the constant `0` or `1`, or
`computed-occlusion` for the geometry-derived occlusion. The channel
count is the number of channels named; an omitted channel is `0`. For example
`--texture-map model-mse.png R=metallic,G=smoothness,B=emissive` reproduces
`--texture mse`, and swapping `G=roughness` writes roughness instead of
smoothness. A packing that names `computed-occlusion` always bakes into an
unwrap layout, as `--texture-map ao.png R=computed-occlusion` does.

## Future work

A later pass may add `--computed-occlusion-radius` and
`--computed-occlusion-falloff` for a sampled neighborhood model that gathers
occluders out to a distance and weights them by a falloff curve, giving smoother
and wider gradients. They do not apply to the current discrete corner method,
which has a fixed one-voxel reach, so they are left out until that model lands.
