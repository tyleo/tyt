# Design notes

*Part of the [Vxl Command-Line Reference](../README.md).*

Rationale for the non-obvious choices, for reviewers.

1. No standalone `optimize`, `pack`, or `unpack`. Every one is a special case
   of `to voxj`, which already owns encoding and container selection, so adding
   them would duplicate that logic and split the invariant that re-encoding
   positions must regenerate samples.
2. `mesh` plus `voxelize` rather than per-format `mesh fbx`. Inferring the mesh
   format from the extension with `--to` and `--from` matches how `to voxj` and
   `to vmax` infer source format, keeps one home for the material options, and
   avoids a subcommand per format. `voxelize` is the conventional verb for the
   inverse.
3. Material maps come from `--texture <name> [path]` for the named presets and
   `--texture-map <path> <channels>` for a custom packing. The presets name the
   common packings, ORM and MSE included, so the common cases are one flag,
   while `--texture-map` expresses any channel-to-attribute packing without a
   code change. Each flag takes its parts as separate arguments, the preset name
   and an optional path, or the output path and the channel list, rather than
   packing a filename, channel count, palette index, and cell list into one
   argument as the original `--texture` flag did. The channel list keeps its
   commas because the RGBA packing is one structured value, and an arity that
   varied with a layout token would need a greedy parser that swallows the
   optional `output` positional. `smoothness` is accepted as the derived
   `1 - roughness`, so it need not be spelled `1-roughness`. The same flags back
   the standalone `material` command so textures can be re-baked without
   re-meshing; both derive the same atlas, so the maps stay aligned to the mesh
   UVs.
4. The material atlas has two layouts, `--atlas palette` and `--atlas unwrap`.
   Palette is the default because it is tiny and, keyed to the palette index
   rather than the per-mesh material set, identical for every mesh on a palette,
   so meshes share one set of maps. Unwrap trades that sharing for a per-mesh UV
   unwrap that can hold spatially varying bakes a single texel per material
   cannot, such as ambient occlusion in the map instead of vertex colors.
5. `mesh` outputs objects as pure geometry, narrowed by object selectors. The
   main use is pulling leaf objects out with no transform data, so selection
   targets objects, by index, the canonical reference, or by a glob over their
   hierarchy path, matched as `hierarchy show` matches node paths, rather than
   baking hierarchy-node subtrees. Index and path are separate repeatable
   options, `--select-index` and `--select`, rather than one option that guesses
   whether a value is an index or a glob, since a name made only of digits is
   unaddressable under that guess. They are flags, not positionals, because the
   optional `output` positional is the house convention and trailing optional
   positionals would be ambiguous. Assembling a placed scene from hierarchy
   nodes, baking transforms and instancing, is a deferred, separate mode.
6. Quantize and remap state their multi-attribute rule. A cell spans every
   attribute, so reducing one attribute has to define what happens to cells
   that share that value but differ elsewhere. Both keep such cells distinct,
   bounding the selected attribute without silently dropping PBR distinctions.
7. Quantize and remap take either a full document or a bare palette JSON, the
   remap `--target` shape. The palette transform is the same either way; a
   document additionally carries voxels, so it can dither the rewritten samples
   in 3D order and narrow that dithering with the object selectors, while a bare
   palette has nothing to walk and skips both. Reusing the selectors keeps one
   addressing model across mesh, material, quantize, and remap.

## Future and nice-to-haves

1. A scene-assembly mode for `mesh` and `material`: selecting hierarchy nodes
   and baking their transforms and instancing into one larger placed mesh,
   complementing the pure-geometry object selectors.
2. stdin and stdout via `-`, so commands compose in pipelines.
3. A dry-run or preview mode for the destructive palette operations.
4. Additional mesh export targets beyond `fbx`, `obj`, and `gltf` as needed.
