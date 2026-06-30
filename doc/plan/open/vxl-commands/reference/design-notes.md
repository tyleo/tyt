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
3. Material maps live on `mesh` as presets plus a `--map` escape hatch. The
   presets name the common packings, ORM and MSE included, so the common cases
   are one flag, while `--map` expresses any custom channel-to-attribute
   packing without a code change. This replaces the original single `--texture`
   flag that packed a filename, a channel count, a palette index, and a
   variadic cell list into one argument. The same map flags are exposed as a
   standalone `material` command so textures can be re-baked without re-meshing;
   both derive the same atlas, so the maps stay aligned to the mesh UVs.
4. `mesh` outputs objects as pure geometry, narrowed by object selectors. The
   main use is pulling leaf objects out with no transform data, so selection
   targets objects, by index, the canonical reference, or by name glob, rather
   than hierarchy nodes. Index and name are separate repeatable options,
   `--select-index` and `--select`, rather than one option that guesses whether
   a value is an index or a glob, since a name made only of digits is
   unaddressable under that guess. They are flags, not positionals, because the
   optional `output` positional is the house convention and trailing optional
   positionals would be ambiguous. Assembling a placed scene from hierarchy
   nodes, baking transforms and instancing, is a deferred, separate mode.
5. Quantize and remap state their multi-attribute rule. A cell spans every
   attribute, so reducing one attribute has to define what happens to cells
   that share that value but differ elsewhere. Both keep such cells distinct,
   bounding the selected attribute without silently dropping PBR distinctions.

## Future and nice-to-haves

1. A scene-assembly mode for `mesh` and `material`: selecting hierarchy nodes
   and baking their transforms and instancing into one larger placed mesh,
   complementing the pure-geometry object selectors.
2. stdin and stdout via `-`, so commands compose in pipelines.
3. A dry-run or preview mode for the destructive palette operations.
4. Additional mesh export targets beyond `fbx`, `obj`, and `gltf` as needed.
