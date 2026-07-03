# Voxj Redesign Implementation Checklist

Tracks porting the codebase from the old voxj format to the redesigned one. Read
the [README](README.md) for the four format deltas, the blast radius, and the
eight closed decisions this checklist encodes. Work top to bottom: the phases
follow the crate dependency order, and nothing compiles end to end until the
chain lands, so expect the workspace to be red between phases and green within a
phase once its downstream is stubbed or updated.

Log non-obvious code-level choices in
[reference/implementation-decisions.md](reference/implementation-decisions.md)
as they land, the same way the vxl-commands plan does.

## Ground rules

- Run `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D
  warnings` before every commit; the pre-commit hook enforces both.
- `voxj` depends only on serde. `voxj-codec` depends on `voxj`. `voxcore`,
  `voxsmith`, and `vxl` stay independent of `tyt-common` and `tyt-injection` and
  use `std::fs`.
- One public type per file, file named in snake_case for the type, private `mod`
  plus flat `pub use` re-export in the parent, per the repo style rules.
- Rebuild each crate's test fixtures in the same phase as its change, not in a
  final sweep. Every existing fixture encodes the old palette shape.
- This is a faithful port: preserve behavior except where a decision deliberately
  changes it (float color default, cell-to-material terminology, non-merging
  layers, metallic default). Capture anything the format newly enables but we are
  not building in the README's deferred-capabilities log.

## Design specifics this checklist assumes

From the closed decisions, so executors do not re-derive them:

- **voxcore is a full pool model with nine kinds.** `json`, `bool`, `float`,
  `int`, `string`, and four canonical color kinds `srgb`, `srgba`, `linear-rgb`,
  `linear-rgba`. Color is stored as float components in natural range: sRGB at
  `0..1` (8-bit hex mapped by dividing by 255), linear allowing above 1 for HDR.
  The wire's six color kinds map onto these four on read; the writer picks a wire
  encoding on write.
- **The wire has eleven kinds.** The color encoding variants (hex for sRGB, float
  for either space) exist only on the wire; voxcore does not carry them. The
  format has no integer color kinds.
- **Layers do not merge.** A consumer wanting one effective material reads a
  selected layer, default the first (index 0). vmax produces a single palette
  whose bindings carry both color and material attributes, one material per
  distinct color-plus-material combination its voxels use.
- **Attribute names are neutral strings** with one shared constant table set to
  the glTF names. The `emissive` split and the `metallicFactor` default flip from
  0 to 1 are adopted.
- **On-wire color defaults to float**, with a `--color-format` option limited to
  `hex` and `float`. Linear colors always serialize as float. min/max come from a
  per-attribute default table (see README Q5).
- **Strictness:** `deny_unknown_fields` on closed structs; the twenty validation
  rules live in `voxj-codec` as named checks; the writer dedups pools by
  `(kind, bounds)` document-wide and dedups identical materials per palette.

## Phase 1: `voxj` data model

- [x] Add `VoxjValuePoolKind`, a closed enum of the eleven wire kinds (`json`,
      `bool`, `float`, `int`, `string`, `srgb-hex`, `srgb-float`, `srgba-hex`,
      `srgba-float`, `linear-rgb-float`, `linear-rgba-float`), serde renamed to
      the kebab-case tags. Unknown tags must fail to deserialize.
- [x] Add a bound type for `min`/`max` that serializes as either a finite JSON
      number or the literal string `"none"`, never JSON `null`. Round-trip both
      forms.
- [x] Add `VoxjValuePool` carrying the kind, its values, and the `min`/`max`
      bounds present only for bounded kinds. Chose the per-kind internally-tagged
      enum with typed values over the struct-with-optional-bounds; recorded in
      the decisions log.
- [x] Add `VoxjPaletteBinding` with `attribute: String` and `pool_ref: usize`
      (serde `poolRef`).
- [x] Rewrite `VoxjPalette` to `bindings: Vec<VoxjPaletteBinding>` and
      `materials: Vec<Vec<usize>>`, column-major, one inner array per binding.
- [x] Add `value_pools: Vec<VoxjValuePool>` to `VoxjRuntimeState` (serde
      `valuePools`), positioned per the spec structure.
- [x] Rename `VoxjObject.palette_refs` to `layer_palette_refs` (serde
      `layerPaletteRefs`) and rewrite its and `voxelSamples`'s doc comments to
      the layer and material-index framing.
- [x] Remove the `serde(default)` on `VoxjObject.origin`; the redesign dropped
      the omitted-defaults-to-zero rule, so origin is now required.
- [x] Add `#[serde(deny_unknown_fields)]` to every closed struct: file, main,
      runtime state, edit state, object, palette, binding, value pool, transform,
      hierarchy node, edit object. Leave `ext` and binding attribute names open.
      (Also on the adjacently-tagged encoding-block enums; serde 1.0.228 honors
      it there, closing the encoding block per spec rule 5.)
- [x] Register the new modules in `lib.rs` and keep `VoxjValue` for the `json`
      pool kind and `ext`.

Gate: `voxj` builds; a hand-written new-shape document round-trips through serde.
Build and round-trip verified; the in-crate round-trip test moves to Phase 2
(`voxj-codec`, which owns `from_voxj_file_bytes` and has `serde_json`) to keep
`voxj` on serde only.

## Phase 2: `voxj-codec`

- [ ] Retarget `voxj_palette_cell_counts` to a material-count helper keyed by
      `layer_palette_refs`, deriving M from a palette's material column length.
      Validation guarantees bindings are non-empty and every column has length
      M at least 1, so no zero-binding fallback is needed; assert rectangularity
      or trust the validator, and record which.
- [ ] Rename `VoxjDecodedObject.palette_refs` to `layer_palette_refs`; reframe
      `samples` docs from cell index to material index. The field type stays
      `Vec<Vec<u32>>`.
- [ ] Rename `cell_counts` to `material_counts` and `num_palettes` to
      `num_layers` across `encode_voxj_object`, `decode_voxj_object`, and
      `encode_voxj_object_optimized`; keep the channel-per-layer arity check and
      note two layers may share a palette. Encode and decode must derive M
      identically.
- [ ] Update `sample_encoding` and `position_encoding` doc comments to the
      material-index and per-layer framing; the codecs themselves do not change.
- [ ] Rewrite `internal/voxj_validation.rs` to the twenty rules: value-pool kind
      recognized and values well-formed per kind, min/max presence and bounds and
      integer-valued and `min <= max`; palette bindings non-empty and distinct
      attribute and `poolRef` in range, materials column count equals bindings
      and equal column length at least 1, every value-index in pool range;
      sample material indices in `[0, M)`; the stricter base64 canonical, pad
      bits zero, rle counts positive and summing to V, hilbert deltas strictly
      positive after the first and bits at most 17, positions in bounds; drop the
      no-duplicate-palette-ref rule. Decide which failures are serde parse errors
      (unknown keys, coercion) versus named checks and record it.
- [ ] Update the named-check list and its doc comment in `check_voxj_file`, plus
      the tests that pin the exact names.
- [ ] Rebuild all fixtures in `validate_voxj_file`, `check_voxj_file`, and
      `from_voxj_file_bytes`; delete the removed-rule tests (duplicate palette
      ref, rgba-attribute format, rectangular rows) and add pool, min/max, kind,
      column-range, and two-layers-one-palette cases.

Gate: `voxj-codec` builds; a new-shape document encodes, decodes, and validates,
and `packed-base64` round-trips at the material-derived width.

## Phase 3: `voxcore`

- [ ] Add the pool model types: `VoxValuePool` (kind, values, bounds),
      `VoxValuePoolKind` (the nine kinds), a `min`/`max` bound type mirroring the
      wire's number-or-none, and `VoxPaletteBinding` (attribute, pool ref). Add
      brand markers as needed (`BVoxValuePool`, and rename the cell brand to a
      material brand).
- [ ] Add the shared value-pool store to `VoxRuntimeState` (an id pool plus
      column, or a plain indexed vec), and extend its clone and `Drop`.
- [ ] Rewrite `VoxPalette` from the attributes-by-cells grid to bindings plus
      column-major materials of value-indices into pools. Rework
      `add_attribute`/`add_cell` into the new build API, `cell_value` into a
      resolve-material-and-attribute-to-value read that hops through
      `materials[b][m]` into the bound pool, and `iter_*`, `remove_*`,
      `clone_palette`, `gc`, and `Drop`. Preserve the unsafe struct-of-arrays
      invariants and Drop ordering exactly.
- [ ] Rename `VoxObject.palette_refs` to layer refs, allow the same palette on
      multiple layers, drop the merge assumption, and change samples from a cell
      id to a material index. Rename `voxel_cell` and friends to material
      terminology and thread the material remap through `gc`.
- [ ] Store colors canonically: a color pool's values are float-component arrays
      in the kind's natural range. Decide whether `VoxValue` gains a dedicated
      color variant or colors ride as `VoxValue::Array` interpreted by the pool
      kind, and record it; the array-plus-kind route is lighter.
- [ ] Update `VoxMain::validate`: drop the duplicate-palette-ref rule, add pool
      kind, min/max presence and bounds, value-well-formed-for-kind, binding
      range, material column arity, and value-index range checks. Add
      `add_value_pool` and iteration accessors. Extend `gc` to compact pools and
      materials.
- [ ] Update `error.rs`: add variants for unknown or invalid kind, missing or
      non-finite bound, malformed value for kind, pool ref out of range, binding
      arity mismatch, and value-index out of range; retire
      `DuplicatePaletteRef`.
- [ ] Update `VoxGcRemap` to a material relabeling plus a value-pool relabeling.
- [ ] Rebuild fixtures and the gc, validate, and remap tests; delete the
      duplicate-palette-ref test.

Gate: `voxcore` builds; build a two-layer object sharing a palette, validate it,
gc it, and read a material's resolved values back.

## Phase 4: `voxsmith` voxj seam

- [ ] Rewrite `vox_palette_from_voxj_palette` to read pools and column-major
      materials into the voxcore pool model, canonicalizing wire color kinds to
      the four voxcore color kinds, and reject duplicate attributes rather than
      last-wins dedup.
- [ ] Rewrite `voxj_palette_from_vox_palette` and `write_voxj` to build wire
      value pools from voxcore pools, dedup by `(kind, bounds)` document-wide,
      emit column-major materials and bindings, apply the `--color-format` choice
      and the per-attribute bounds table, and dedup identical materials.
- [ ] Rework `vox_value_from_voxj_value` and `voxj_value_from_vox_value` into
      kind-directed decode and encode, mapping color hex, int, and float to and
      from the canonical float form and validating each value against its kind.
- [ ] Update the decoded-object seam
      (`vox_object_from_voxj_decoded_object`, `voxj_decoded_object_from_vox_object`)
      for layer refs and material-index samples, relying on voxcore now allowing
      duplicate palette refs.
- [ ] Point callers at the retargeted material-count helper.
- [ ] Rebuild the `from_voxj_file` fixtures; add two-layers-one-palette and
      min/max-violation cases and a `voxj -> vox -> voxj` round-trip that
      accounts for color canonicalization and the float default.

Gate: a new-shape `.voxj` loads into `VoxMain` and writes back, stable under the
color and pool normalization.

## Phase 5: `voxsmith` color helpers and glTF pipeline

- [ ] Move the shared attribute-name constants into one module set to the glTF
      names, imported by every converter, the glTF pipeline, and vxl.
- [ ] Retarget `object_color_ref` to `baseColorFactor`, and generalize
      `cell_color` and `parse_color_hex` to resolve a color through
      material-to-binding-to-pool and decode by pool kind, not hex only.
- [ ] Rename `MeshMaterial` fields and `MATERIAL_ATTRIBUTES` to the glTF vocab,
      split `emissive` into `emissiveFactor` color and `emissiveStrength` number,
      and set the default-scalar table to the new names with `metallicFactor`
      default 1. Keep writing `metallicFactor` explicitly where voxelize does so
      the matte look is preserved; the flip only bites absent-attribute defaults.
- [ ] Redefine a material for the atlas as a material index into the selected
      layer (default 0) in `used_materials`; remove the cross-reference merge.
- [ ] Carry emissive as color plus strength through `sample_material`,
      `mesh_emissive_map`, and the `EmissiveColor` bake
      (`emissiveFactor * emissiveStrength` in linear); keep import and export in
      sync for glTF round-trips.
- [ ] Update the glTF import (`from_gltf_bytes`) and its tests, and the atlas and
      material-document tests, for the renamed attributes and the color default.

Gate: glTF export and import build and round-trip; a voxelized flat mesh renders
as before.

## Phase 6: `voxsmith` goxl, mvox, qbcl, vmax converters

- [ ] goxl: build a color pool bound to `baseColorFactor`, emit materials as
      value-indices, and read color back through the pool-kind helper both in the
      direct and synthesized paths.
- [ ] qbcl (qb, qbt, qbcl): replace the alpha-less `rgb` attribute with a color
      pool bound to `baseColorFactor`; decide the alpha convention for a
      three-channel source (canonical `srgb`, alpha synthesized as 1 where a
      four-channel consumer needs it) and apply it uniformly across the three
      near-duplicate files.
- [ ] mvox: map color to a color pool, the material scalars to float pools, and
      the type token to a string pool; represent absent optionals with the
      attribute's default value rather than null, since pools are non-null; keep
      the color-index and material.id coupling as the material index.
- [ ] vmax: fold the color palette and material palette into one palette with
      bindings for `baseColorFactor` and the material attributes over separate
      pools, one material per distinct color-plus-material combination the voxels
      use; keep `shadows` and `absorption` as custom attributes; on write-back
      reconstruct the distinct color and material sets and each voxel's original
      color and material indices.
- [ ] Update `voxelize_mesh` to build pools, bindings, and column-major materials
      instead of `add_attribute` plus `add_cell`, and fold the split emissive into
      its material key.
- [ ] Rebuild every converter's fixtures and keep the byte-exact round-trip tests
      passing by making pool ordering, dedup, and color-kind choice deterministic;
      regenerate goldens where the float default or the vmax fold changes bytes.

Gate: all five converters build; each format round-trips through the new voxcore
model.

## Phase 7: `vxl`

- [ ] Rename user-facing cell terminology to material: `--show-cells`, the
      `cells` and `cellCount` columns and JSON keys, and the cell wording in
      `palette show`, `palette list`, `info`, and `hierarchy show`.
- [ ] Rework `palette show` color rendering to read the bound pool's kind rather
      than sniffing Text versus Number; handle three-component colors without
      alpha and non-hex encodings; round-trip pooled array, int, float, and bool
      values in the JSON and text layouts instead of collapsing them to null.
- [ ] Replace the binary `AttributeType` (Scalar or Color) with a
      kind-and-bounds aware model, and let `ColorComponent` address
      three-component colors and float or int components.
- [ ] Update `mesh` and the texture presets to the glTF vocab: `baseColorFactor`
      is the color, `emissiveFactor` is also a color, and the presets map
      `metallic`, `roughness`, `occlusion`, and `emissive` to their new names;
      split `EmissiveColor` into `emissiveFactor` times `emissiveStrength`.
- [ ] Give `--define-attribute` and `--texture-map` a layer selector on `mesh`
      and `material`, defaulting to the first layer, replacing the merge
      assumption in `attribute_binding`.
- [ ] Add a `--color-format` option (`hex` or `float`, default float) to the
      voxj-writing path, following the existing `color_format` utility pattern.
- [ ] Update `voxj_sample_encoding` docs to the per-layer material-index framing,
      point `max_palette_cells` at the material count, and route `fill_color`
      into pool population.
- [ ] Update `validate` output and tests for the new check names; rebuild the
      palette, info, and hierarchy fixtures.

Gate: `vxl` builds; `info`, `palette list`, `palette show`, `validate`, `mesh`,
and `to voxj` work end to end on a new-shape document.

## Phase 8: docs

- [ ] Update `doc/plan/open/vxl-commands` so its palette model, the `--attribute`
      default, the merge-to-layer-selection change, the cell-to-material rename,
      and the new `--color-format` and `--layer` options match the code.
- [ ] Confirm the format spec and every crate README describe the shipped
      behavior, and move the plan to `doc/plan/` closed status when done.
