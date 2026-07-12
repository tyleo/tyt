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
3. Material maps come from `--texture <preset>` for the named presets and bundles
   and `--texture-map <file-name> <channels>` for a custom packing. The presets
   name the common packings, ORM and MSE included, so the common cases are one
   flag, while `--texture-map` expresses any channel-to-attribute packing without
   a code change. No flag packs several tokens into one quoted string: `--texture`
   is a plain, repeatable value enum clap validates and completes, with a map's
   file name split out to `--texture-name <preset> <file-name>` and
   `--texture-name-prefix`, and `--texture-map` takes its file name and channel
   list as two arguments, rather than packing a filename, channel count, palette
   index, and material list into one argument as the original `--texture` flag
   did. The channel list keeps its commas because the RGBA packing is one
   structured value, and an arity that varied with a layout token would need a
   greedy parser that swallows the optional `output` positional. `smoothness` is
   accepted as the derived `1 - roughnessFactor`, so it need not be spelled
   `1-roughnessFactor`. The same flags back
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
6. Quantize, remap, and `voxelize`'s `--max-palette-materials` share one reduction
   rule: material follows color. Reducing the compared attribute
   (`baseColorFactor` by default) clusters materials by it and collapses each
   cluster to one representative material, so a material's other attributes ride
   along with its color and a count bounds the material count, not just the
   attribute's distinct values. The earlier rule kept materials that share a color
   but differ in their other attributes distinct; it was dropped so a count
   actually bounds the palette, the representative stays a real material rather
   than an average, and the three commands share one engine and its `--method` /
   `--space` / `--dither` controls. The accepted cost is that fusing two colors
   fuses their materials too.
7. Quantize and remap take either a full document or a bare palette JSON, the
   remap `--target` shape. The palette transform is the same either way; a
   document additionally carries voxels, so it can dither the rewritten samples
   in 3D order and narrow that dithering with the object selectors, while a bare
   palette has nothing to walk and skips both. Reusing the selectors keeps one
   addressing model across mesh, material, quantize, and remap.
8. Custom attributes reach `--texture-map` and `--vertex-map` through a declared
   binding, `--define-attribute <name>=<key>[:<type>]`, rather than inline
   qualifiers. The voxel-json format stores attributes generically, so a packing
   must read a key the presets do not name, of a type the tool cannot infer: a
   custom value may be a `0..1` number or a `#RRGGBBAA` color, and only a color
   exposes `r`/`g`/`b`/`a` components. A binding states the type once and gives
   the source a name, so the packing grammar stays a flat `name`, `1-name`, or
   `name.component`, and the name is reusable across channels, images, and vertex
   attributes. It shadows a built-in on collision, scoped to the custom packings
   so a binding never silently changes a `--texture` or `--vertex` preset. The
   binding reads the key from the meshed layer's material, the layer `mesh`
   selects with `--layer` and the first by default, the same layer the rest of
   `mesh` bakes, rather than naming a separate source. Layers no longer merge, so
   each layer is one palette and every voxel samples one material per layer;
   reaching another layer's value is just another `--layer`. The built-in
   `baseColorFactor` is itself a color, so `baseColorFactor.a` and an RGBA split
   need no binding,
   complementing the whole-color `albedo` preset. Inline `N:` and `.component`
   qualifiers with no declaration were dropped: they cannot carry an explicit
   type, leaving an ambiguous custom value no home, and they give no reusable
   name.
9. `palette show` reads the attribute type from its bound value pool's kind, a
   color kind for a color and a scalar kind for a number, where mesh's
   `--define-attribute` must be told the type. The difference is that `show`
   reads concrete materials whose pool names its own kind, while a `--texture-map`
   packing is compiled before any material is read and cannot. `--type` stays as an
   optional override so a preview can assert the same type a `--define-attribute`
   binding declares and read a custom key exactly as the mesh packing will. The
   `.component` grammar is reused from `--texture-map`, so `baseColorFactor.a`
   means one thing across show, mesh, and the packings; it is read-only inspection sugar,
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
    factors into one palette material per material, exact and tiny for stylized
    meshes;
    per-texel samples the maps at each voxel's surface point, area-averaged so
    fine texture does not alias into a muddy palette; `flat` ignores the mesh for
    a one-color body. Per-primitive is not a rival of per-texel but a part of it:
    any sampling first attributes a voxel to a material and reads its factors, and
    a `solid` body's interior voxels sit on no surface and fall back to that
    factor regardless, so the two modes share one path and per-texel only adds the
    texel sampler. `--fill-color` is the color the modes cannot sample: the whole
    body under `flat`, white when omitted, or only the invented interior of a
    `solid` per-* body, leaving the sampled exterior alone; a set color is rejected
    for a hollow shell, which is all surface. `auto` engages per-texel on textured
    meshes because importing a textured model implies wanting its surface color,
    while the explicit modes override that guess. The flat color mode is named
    `flat`, not `solid`, so it does not collide with `--fill-mode solid`, which is
    geometry.
11. `voxelize --max-palette-materials` bounds the generated palette, defaulting to 256 (a
    one-byte sample index and the familiar color ceiling). It auto-reduces with a
    warning rather than erroring or truncating, since a textured mesh exceeding
    the cap is the normal case, and reuses the `palette quantize` engine and its
    `--method` / `--space` / `--dither` controls rather than inventing its own, so
    the inline cap and the standalone command cannot diverge; `none` disables it
    for bit-exact materials. Sampling drops no PBR: voxelize writes the same
    `baseColorFactor`, `metallicFactor`, `roughnessFactor`, `emissiveFactor`,
    `emissiveStrength`, and `occlusionStrength` attributes `mesh` bakes, so the
    two are inverses.
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
    per-vertex metallic, roughness, and the palette presets go in
    application-specific `_NAME` attributes only a custom shader reads. The
    palette indirection comes in two shapes because an object keeps a palette
    per layer, which no single index spans: `palette-index` flattens to the
    distinct materials of the layer `mesh` bakes, one `_PALETTEINDEX` into a
    per-mesh table, the smallest carrier; `palette-layers` keeps every layer, one
    `_PALETTEINDEXn` per layer plus each layer's palette, summing rather than
    multiplying the layer sizes, staying shareable across meshes, and alone
    preserving every layer's material rather than only the baked one. A product
    index over every layer combination was rejected: `palette-layers` is both
    smaller and shareable, so the product never wins. The texture
    `--atlas palette` keys on the baked layer's palette instead, since per-layer
    textures would need a custom shader and forfeit the atlas's portability, so
    per-layer material lives on the vertex carrier.
13. Texture images and the palette data each store `embedded`, `external`, or
    `both`, with a format-driven default: `embedded` for a `.glb`, `external`
    for a `.gltf`, so zero-config output matches what each glTF form normally
    carries while either can be forced. `embedded` keeps one shippable file (a
    `.glb` chunk or a `.gltf` data URI for images, the palette under
    `extras.vxl`); `external` writes loose `.png` and `-palette.json` files that
    are easier to iterate on; `both` embeds the copy the mesh references and also
    drops the loose files as working sources, so a deliverable stays
    self-contained while its textures stay editable. The two resources take
    separate flags, `--texture-storage` and `--palette-storage`, because the
    common workflow keeps textures external for tweaking while the palette data
    rides inside the mesh. The palette data is plain JSON, not a binary buffer,
    for the same reasons voxel-json is plain JSON: openable and diffable, and a
    loader parses it once before packing it into a uniform buffer or palette
    texture; the same JSON object is the `extras.vxl` value and the sidecar
    file's whole content.

## Future and nice-to-haves

1. A scene-assembly mode for `mesh` and `material`: selecting hierarchy nodes
   and baking their transforms and instancing into one larger placed mesh,
   complementing the pure-geometry object selectors.
2. stdin and stdout via `-`, so commands compose in pipelines.
3. A dry-run or preview mode for the destructive palette operations.
4. Additional mesh formats beyond glTF, such as `fbx` and `obj`, as needed, for
   both `mesh` output and `voxelize` input.
5. Gitignore-style multiple patterns shipped for `hierarchy show`: an ordered
   list of select and deselect patterns with `!` negation, trailing-slash
   node-only matching, and last-match-wins, so a selection subtracts as well as
   adds. It is built on a new dependency-light `pathspec` crate, a Rust port
   of the C# `com.tyleo.gitignore` package layered on the `globset` already in
   use, rather than the heavier `ignore` crate. The parent-directory rule was
   settled git-faithful: an excluded node prunes its subtree, matching the
   reference engine. The `--select` object selectors will inherit the same engine
   when they land. See
   [implementation decisions](implementation-decisions.md#gitignore-style-pattern-matching).
