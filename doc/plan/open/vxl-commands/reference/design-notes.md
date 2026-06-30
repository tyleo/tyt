# Design notes

*Part of the [Vxl Command-Line Reference](../README.md).*

Rationale for the non-obvious choices, for reviewers.

1. No standalone `optimize`, `pack`, or `unpack`. Every one is a special case
   of `to voxj`, which already owns encoding and container selection, so adding
   them would duplicate that logic and split the invariant that re-encoding
   positions must regenerate samples.
2. `mesh` plus `voxelize` rather than per-format `mesh gltf`. Inferring the mesh
   format from the extension with `--to` and `--from` matches how `to voxj` and
   `to vmax` infer source format, keeps one home for the material options, and
   leaves room for more mesh formats without a subcommand per format. glTF is the
   only mesh format for now. `voxelize` is the conventional verb for the inverse.
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
5. `mesh` outputs an object as pure geometry, narrowed by object selectors. The
   main use is pulling leaf objects out with no transform data, so selection
   targets objects, by index, the canonical reference, or by a glob over the
   hierarchy path, matched as `hierarchy show` matches node paths so a node path
   selects its subtree. Index and path are separate repeatable options,
   `--select-index` and `--select`, rather than one option that guesses whether a
   value is an index or a glob, since a name made only of digits is unaddressable
   under that guess. They are flags, not positionals, because the optional
   `output` positional is the house convention and trailing optional positionals
   would be ambiguous. `mesh` errors when the selection is not exactly one object;
   how to output several, and whether to bake a node's subtree, transforms, and
   instancing into one placed mesh, is a deferred, separate mode.
6. Quantize, remap, and `voxelize`'s `--max-palette` share one reduction rule:
   material follows color. Reducing the compared attribute (`rgba` by default)
   clusters cells by it and collapses each cluster to one representative cell, so
   a cell's other attributes ride along with its color and a count bounds the
   cell count, not just the attribute's distinct values. The earlier rule kept
   same-color/different-material cells distinct; it was dropped so a count
   actually bounds the palette, the representative stays a real cell rather than
   an average, and the three commands share one engine and its `--method` /
   `--space` / `--dither` controls. The accepted cost is that fusing two colors
   fuses their materials too.
7. Quantize and remap take either a full document or a bare palette JSON, the
   remap `--target` shape. The palette transform is the same either way; a
   document additionally carries voxels, so it can dither the rewritten samples
   in 3D order and narrow that dithering with the object selectors, while a bare
   palette has nothing to walk and skips both. Reusing the selectors keeps one
   addressing model across mesh, material, quantize, and remap.
8. Custom attributes reach `--texture-map` through a declared binding,
   `--define-attribute <name> <palette-index> <key> [type]`, rather than inline
   qualifiers. The voxel-json format stores attributes generically, so a packing
   must read a key the presets do not name, of a type the tool cannot infer: a
   custom value may be a `0..1` number or a `#RRGGBBAA` color, and only a color
   exposes `r`/`g`/`b`/`a` components. A binding states the type once and gives
   the source a name, so the packing grammar stays a flat `name`, `1-name`, or
   `name.component`, and the name is reusable across channels and images. It
   shadows a built-in on collision, scoped to `--texture-map` so a binding never
   silently changes a `--texture` preset. The palette index is a position in the
   object's `paletteRefs`, the merge order `mesh` already walks, always valid for
   the single object `mesh` outputs and the way to disambiguate a key shared by
   several layers. The built-in `rgba` is itself a color, so `rgba.a` and an
   RGBA split need no binding, complementing the whole-color `albedo` preset.
   Inline `N:` and `.component` qualifiers with no declaration were dropped: they
   cannot carry an explicit type, leaving an ambiguous custom value no home, and
   they give no reusable name.
9. `palette show` infers the attribute type from the stored value, a `#RRGGBBAA`
   string for a color and a number for a scalar, where mesh's
   `--define-attribute` must be told the type. The difference is that `show`
   reads concrete cells and the value names its own type, while a `--texture-map`
   packing is compiled before any cell is read and cannot. `--type` stays as an
   optional override so a preview can assert the same type a `--define-attribute`
   binding declares and read a custom key exactly as the mesh packing will. The
   `.component` grammar is reused from `--texture-map`, so `rgba.a` means one
   thing across show, mesh, and the packings; it is read-only inspection sugar,
   scoped to show, so the mutating palette commands keep whole-attribute
   semantics. `auto` keeps numeric output for scalars and swatches only true
   colors, beside their hex. `swatch` and `swatch-value` extend swatches to
   scalars and extracted channels as a grayscale ramp, since a single `0..1`
   value renders as gray, the first printing the swatch alone and the second
   adding the exact hex or number beside it, while `value` drops the swatch for
   piping.
10. `voxelize` separates geometry from color. `--fill-mode` chooses a filled body
    or a hollow shell; `--material-mode` chooses where color comes from and
    defaults to `auto`, sampling `per-texel` when the mesh is textured and
    `per-primitive` when it is not. Per-primitive reads each material's flat
    factors into one cell per material, exact and tiny for stylized meshes;
    per-texel samples the maps at each voxel's surface point, area-averaged so
    fine texture does not alias into a muddy palette; `flat` ignores the mesh for
    a one-color body. Per-primitive is not a rival of per-texel but a part of it:
    any sampling first attributes a voxel to a material and reads its factors, and
    a `solid` body's interior voxels sit on no surface and fall back to that
    factor regardless, so the two modes share one path and per-texel only adds the
    texel sampler. `--fill-color` is the color the modes cannot sample: the whole
    body under `flat`, where `none` yields white, or only the invented interior of a
    `solid` per-* body, leaving the sampled exterior alone; it does nothing for a
    hollow shell, which is all surface. `auto` engages per-texel on textured
    meshes because importing a textured model implies wanting its surface color,
    while the explicit modes override that guess. The flat color mode is named
    `flat`, not `solid`, so it does not collide with `--fill-mode solid`, which is
    geometry.
11. `voxelize --max-palette` bounds the generated palette, defaulting to 256 (a
    one-byte sample index and the familiar color ceiling). It auto-reduces with a
    warning rather than erroring or truncating, since a textured mesh exceeding
    the cap is the normal case, and reuses the `palette quantize` engine and its
    `--method` / `--space` / `--dither` controls rather than inventing its own, so
    the inline cap and the standalone command cannot diverge; `none` disables it
    for bit-exact materials. Sampling drops no PBR: voxelize writes the same
    `rgba`, `metallic`, `roughness`, `emissive`, and `occlusion` attributes
    `mesh` bakes, so the two are inverses.
12. Vertex attribute maps share the texture maps' source grammar and add only a
    carrier. A map resolves a material value the same way whether it lands in a
    texel or on a vertex, so `--vertex` and `--vertex-map` reuse the
    `--texture`/`--texture-map` presets, the `--texture-map` channel grammar, and
    `--define-attribute`, differing only in destination. This folds the former
    `--vertex-computed-occlusion` boolean into `--vertex computed-occlusion`, one
    cell of the general family, and leaves the texture presets untouched. A
    separate `--atlas vertex` was rejected: atlas is a texture layout, while the
    carrier is chosen per map by `--texture` versus `--vertex`, so color can ride
    on vertices while PBR bakes to a shared atlas in the same run. `COLOR_0` is
    glTF's only per-vertex PBR slot, so per-vertex color is portable while
    per-vertex metallic, roughness, and `palette-index` go in
    application-specific `_NAME` attributes only a custom shader reads. The
    `palette-index` attribute is the palette atlas's indirection with the index
    on the vertex and the per-index material table in `extras` rather than a UV
    into a texture, the most compact carrier and the exact-value alternative to
    the palette atlas.

## Future and nice-to-haves

1. A scene-assembly mode for `mesh` and `material`: selecting hierarchy nodes
   and baking their transforms and instancing into one larger placed mesh,
   complementing the pure-geometry object selectors.
2. stdin and stdout via `-`, so commands compose in pipelines.
3. A dry-run or preview mode for the destructive palette operations.
4. Additional mesh formats beyond glTF, such as `fbx` and `obj`, as needed, for
   both `mesh` output and `voxelize` input.
