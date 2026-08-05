# Design notes

*Part of the [Vxl Command-Line Reference](../README.md).*

Rationale for the non-obvious choices, for reviewers. The `mesh` command's
design lives in [mesh](mesh.md).

1. No standalone `optimize`, `pack`, or `unpack`. Every one is a special case
   of `to voxj`, which already owns encoding and container selection, so adding
   them would duplicate that logic and split the invariant that re-encoding
   positions must regenerate samples.
2. `mesh` plus `voxelize` rather than per-format `mesh gltf`. Inferring the mesh
   format from the extension with `--to` and `--from` matches how `to voxj` and
   `to vmax` infer source format, keeps one home for the material options, and
   leaves room for more mesh formats without a subcommand per format. glTF is the
   only mesh format for now. `voxelize` is the conventional verb for the inverse.
3. Quantize, remap, and `voxelize`'s `--max-palette-materials` share one reduction
   rule: material follows color. Reducing the compared property
   (`baseColor` by default) clusters materials by it and collapses each
   cluster to one representative material, so a material's other properties ride
   along with its color and a count bounds the material count, not just the
   property's distinct values. The earlier rule kept materials that share a color
   but differ in their other properties distinct; it was dropped so a count
   actually bounds the palette, the representative stays a real material rather
   than an average, and the three commands share one engine and its `--method` /
   `--space` / `--dither` controls. The accepted cost is that fusing two colors
   fuses their materials too.
4. Quantize and remap take either a full document or a bare palette JSON, the
   remap `--target` shape. The palette transform is the same either way; a
   document additionally carries voxels, so it can dither the rewritten samples
   in 3D order and narrow that dithering with the object selectors, while a bare
   palette has nothing to walk and skips both. Reusing the selectors keeps one
   addressing model across mesh, material, quantize, and remap.
5. `palette show` reads a property's meaning from its name, per the format's
   glTF vocabulary; a value pool carries only a shape, the same rule the
   [`mesh` packings](mesh.md#channel-expressions) bake by. The selector's
   reading field names the transfer and spelling explicitly (`auto`,
   `linear-float`, `plain`, `srgb-float`, `srgb-hex`); under `auto` a
   vocabulary color name reads `srgb-hex`, a vocabulary scalar reads its
   number, and a custom key reads `plain`, so a custom vector asserts
   color only through an explicit color reading. A property absent from a
   palette follows the format's unbound-default rule: a glTF built-in takes
   its spec default, a custom key is an error. The `.component` grammar is
   shared with the mesh channel expressions through either alias set
   (`.r`/`.g`/`.b`/`.a` or `.x`/`.y`/`.z`/`.w`), so `baseColor.a` means one
   thing across show, mesh, and the packings; it is read-only inspection
   sugar, scoped to show, so the mutating palette commands keep
   whole-property semantics. `auto` keeps numeric output for scalars and
   swatches only vocabulary colors, beside their hex. `swatch` and
   `swatch-value` extend swatches to scalars and extracted components as a
   grayscale ramp, since a single `0..1` value renders as gray, the first
   printing the swatch alone and the second adding the exact reading beside
   it, while `value` drops the swatch for piping.
6. `voxelize` separates geometry from color. `--fill-mode` chooses a filled body
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
7. `voxelize --max-palette-materials` bounds the generated palette, defaulting to 256 (a
   one-byte sample index and the familiar color ceiling). It auto-reduces with a
   warning rather than erroring or truncating, since a textured mesh exceeding
   the cap is the normal case, and reuses the `palette quantize` engine and its
   `--method` / `--space` / `--dither` controls rather than inventing its own, so
   the inline cap and the standalone command cannot diverge; `none` disables it
   for bit-exact materials. Sampling drops no PBR: voxelize writes the same
   `baseColor`, `metallic`, `roughness`, `emissiveColor`,
   `emissiveStrength`, and `occlusionStrength` properties `mesh` bakes, so the
   two are inverses.

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
