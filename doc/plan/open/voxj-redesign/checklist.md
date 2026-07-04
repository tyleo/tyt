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

- [x] Retarget `voxj_palette_cell_counts` to a material-count helper keyed by
      `layer_palette_refs`, deriving M from a palette's material column length.
      Validation guarantees bindings are non-empty and every column has length
      M at least 1, so no zero-binding fallback is needed; assert rectangularity
      or trust the validator, and record which. (Renamed to
      `voxj_palette_material_counts`; trusts the validator; M is the first
      column's length. Recorded in the decisions log.)
- [x] Rename `VoxjDecodedObject.palette_refs` to `layer_palette_refs`; reframe
      `samples` docs from cell index to material index. The field type stays
      `Vec<Vec<u32>>`.
- [x] Rename `cell_counts` to `material_counts` and `num_palettes` to
      `num_layers` across `encode_voxj_object`, `decode_voxj_object`, and
      `encode_voxj_object_optimized`; keep the channel-per-layer arity check and
      note two layers may share a palette. Encode and decode must derive M
      identically. (Both call the one shared helper.)
- [x] Update `sample_encoding` and `position_encoding` doc comments to the
      material-index and per-layer framing; the codecs themselves do not change.
      (`sample_encoding` reframed; `position_encoding` needed none, it carries no
      palette or material framing.)

Chunk split (see decisions log): items 1 to 4 above and the STRUCTURAL parts of
5 to 7 below landed in one chunk. The value-pool CONTENT validation and the
stricter block-internal rules and their fixtures are deferred to a follow-up
chunk, so items 5 to 7 stay unchecked until that lands.

- [x] Rewrite `internal/voxj_validation.rs` to the twenty rules: value-pool kind
      recognized and values well-formed per kind, min/max presence and bounds and
      integer-valued and `min <= max`; palette bindings non-empty and distinct
      attribute and `poolRef` in range, materials column count equals bindings
      and equal column length at least 1, every value-index in pool range;
      sample material indices in `[0, M)`; the stricter base64 canonical, pad
      bits zero, rle counts positive and summing to V, hilbert deltas strictly
      positive after the first and bits at most 17, positions in bounds; drop the
      no-duplicate-palette-ref rule. Decide which failures are serde parse errors
      (unknown keys, coercion) versus named checks and record it.
      (Chunk 1 done: `check_palettes` rewritten to bindings non-empty / distinct
      attribute / `poolRef` in range / materials column arity / equal column
      length >= 1 / value-index in pool range; `check_geometry` sample-material
      indices in `[0, M)`; `check_indices` drops the duplicate-palette-ref rule
      and keeps the layer-ref range check. Chunk 2 done: value-pool content rules
      (rule 9) landed as the new `value-pools` check in `check_value_pools.rs`:
      values non-empty, int/float value within min/max, integer-valued int
      bounds, `min <= max`, hex-color pattern, and float color-component ranges.
      Chunk 3 done: the block-internal rules (11.2, 11.3, 13.2, 13.3, 14) landed
      by tightening the decode path, so they report through the existing
      `blocks`, `unique-positions`, and `bounds` checks with no new name. rle
      streams reject an odd length and a zero count (11.2); packed and bitmap
      blocks require the exact byte count and zero pad bits (11.3, 13.2); the
      hilbert path caps `bits` at 17 and errors on a truncated or overlong
      varint (13.3.2, 13.3.1), while a non-positive delta collapses to a
      repeated Hilbert index that `unique-positions` catches; base64
      canonicality (rule 14) was already enforced by the base64 STANDARD engine.
      All twenty rules are now covered.)
- [x] Update the named-check list and its doc comment in `check_voxj_file`, plus
      the tests that pin the exact names. (Chunk 1 done: `sample-cells` renamed
      to `sample-materials`; `palettes`/`indices` docs and the name-list test
      updated. Chunk 2 done: `value-pools` added between `version` and `palettes`
      in `Check`, `REPORT_ORDER` (now 13), the `check_voxj_file` doc list, and the
      `reports_every_check_in_order` name-list test. The deferred block-internal
      rules report through the existing `blocks`/`bounds`/`unique-positions`
      checks via a tighter decode, so they add no new name.)
- [x] Rebuild all fixtures in `validate_voxj_file`, `check_voxj_file`, and
      `from_voxj_file_bytes`; delete the removed-rule tests (duplicate palette
      ref, rgba-attribute format, rectangular rows) and add pool, min/max, kind,
      column-range, and two-layers-one-palette cases. (Chunk 1 done: fixtures
      rebuilt to the value-pool/layer/material shape; removed-rule tests deleted;
      value-index-range, poolRef-range, ragged-column, column-count-mismatch,
      duplicate-binding-attribute, and two-layers-one-palette cases added.
      Chunk 2 done: `validate_voxj_file` gains empty-pool, value-below-min,
      value-above-max, min>max, non-integer-int-bound, malformed-hex,
      lowercase-hex, srgb-component-range, linear-component-range rejection cases
      plus an unbounded/HDR acceptance case, and `check_voxj_file` gains a
      report-level `value-pools` failure case. Chunk 3 done: decode-level cases
      in `decode_voxj_object` and `decode_varint` (wrong bitmap/packed byte
      count, non-zero bitmap/packed pad bits, odd-length and zero-count rle,
      oversized hilbert grid, truncated and overlong varint) and validate-level
      cases (those plus non-canonical base64 and a zero-delta hilbert caught as a
      duplicate position), valid-block positive controls for bitmap and hilbert
      positions, and a `blocks` report-level case in `check_voxj_file`.)

Gate: `voxj-codec` builds; a new-shape document encodes, decodes, and validates,
and `packed-base64` round-trips at the material-derived width.

## Phase 3: `voxcore`

- [x] Add the pool model types: `VoxValuePool` (kind, values, bounds),
      `VoxValuePoolKind` (the nine kinds), a `min`/`max` bound type mirroring the
      wire's number-or-none, and `VoxPaletteBinding` (attribute, pool ref). Add
      brand markers as needed (`BVoxValuePool`, and rename the cell brand to a
      material brand).
      (Chunk 1 done: the additive leaf types landed as `VoxValuePoolKind` (nine
      kinds), `VoxBound` (number-or-none, no serde), `VoxValuePool` (a per-kind
      discriminated union with exact typed values mirroring the wire's
      `VoxjValuePool`; `min`/`max` only on the `float`/`int` variants; colors as
      `Vec<[f64; N]>`), `VoxPaletteBinding` (`attribute`,
      `pool: U32Id<BVoxValuePool>`), and the `BVoxValuePool` brand, all registered
      in `lib.rs`. Chunk 3 (palette rewrite) done: `BVoxPaletteCell` renamed to
      `BVoxMaterial`, and alongside it `BVoxAttribute` -> `BVoxPaletteBinding` and
      `BVoxPaletteRef` -> `BVoxLayer` so the brands match the new model. See the
      decisions log.)
- [x] Add the shared value-pool store to `VoxRuntimeState` (an id pool plus
      column, or a plain indexed vec), and extend its clone and `Drop`.
      (Chunk 2 done: `value_pool_ids: IdStruct<BVoxValuePool>` plus
      `value_pools: IdField<BVoxValuePool, VoxValuePool>`, the id-pool-plus-column
      shape the other stores use so bindings resolve and gc can relabel pools;
      `clone_runtime_state` and `Drop` extended. The additive `VoxMain`
      value-pool accessors from item 6 landed alongside it, so the store is
      reachable and tested. See the decisions log.)
- [x] Rewrite `VoxPalette` from the attributes-by-cells grid to bindings plus
      column-major materials of value-indices into pools. Rework
      `add_attribute`/`add_cell` into the new build API, `cell_value` into a
      resolve-material-and-attribute-to-value read that hops through
      `materials[b][m]` into the bound pool, and `iter_*`, `remove_*`,
      `clone_palette`, `gc`, and `Drop`. Preserve the unsafe struct-of-arrays
      invariants and Drop ordering exactly.
      (Chunk 3 done: `bindings: IdField<BVoxPaletteBinding, VoxPaletteBinding>`
      plus material-major `materials: IdField<BVoxMaterial, IdField<
      BVoxPaletteBinding, u32>>`; `add_binding`/`add_material` build API;
      `value_index` returns the raw index and `VoxMain::material_value` resolves
      it through the bound pool; `remove_binding`/`remove_material`/`gc`/`Drop`
      simplified because value-indices are Copy. Verified leak- and UB-clean
      under Miri. See the decisions log.)
- [x] Rename `VoxObject.palette_refs` to layer refs, allow the same palette on
      multiple layers, drop the merge assumption, and change samples from a cell
      id to a material index. Rename `voxel_cell` and friends to material
      terminology and thread the material remap through `gc`.
      (Chunk 3 done: `BVoxPaletteRef` -> `BVoxLayer`, `layer_palettes`/`samples`
      keyed by layer, `add_layer`/`voxel_material`/`iter_layers`/`layer_count`/
      `remove_layer`/`remove_layers_to`/`repaint_material`; samples are
      `U32Id<BVoxMaterial>`; no duplicate-layer rule.)
- [x] Store colors canonically: a color pool's values are float-component arrays
      in the kind's natural range. Decide whether `VoxValue` gains a dedicated
      color variant or colors ride as `VoxValue::Array` interpreted by the pool
      kind, and record it; the array-plus-kind route is lighter.
      (Settled in chunk 1: neither route; colors are exact `Vec<[f64; N]>` in the
      four color variants of `VoxValuePool`, so `VoxValue` backs only `json` and
      `ext`. See the decisions log.)
- [x] Update `VoxMain::validate`: drop the duplicate-palette-ref rule, add pool
      kind, min/max presence and bounds, value-well-formed-for-kind, binding
      range, material column arity, and value-index range checks. Add
      `add_value_pool` and iteration accessors. Extend `gc` to compact pools and
      materials.
      (The `add_value_pool` and iteration accessors landed early with the
      value-pool store, chunk 2. Chunk 3 done: `validate` gained the value-pool
      content check (non-empty, bounds finite/ordered/integer-valued, values in
      bounds, color components in range), binding pool-ref resolution, duplicate
      binding attribute, and material value-index range; column arity is
      structural (the material build API retains one value-index per binding), so
      no runtime arity check is needed. `gc` now compacts the value-pool store,
      relabels every binding pool-ref, and compacts materials.)
- [x] Update `error.rs`: add variants for unknown or invalid kind, missing or
      non-finite bound, malformed value for kind, pool ref out of range, binding
      arity mismatch, and value-index out of range; retire
      `DuplicatePaletteRef`.
      (Chunk 3 done: added `EmptyPool`, `PoolBound` (non-finite / non-integer /
      unordered), `PoolValue` (malformed value or out of bounds), `BindingPool`,
      `MaterialValue`; renamed `SampleCell` -> `SampleMaterial` and
      `DuplicateAttribute` -> `DuplicateBindingAttribute`; dropped
      `DuplicatePaletteRef`. Kind is a typed enum, so there is no unknown-kind
      variant; arity is structural, so there is no arity variant.)
- [x] Update `VoxGcRemap` to a material relabeling plus a value-pool relabeling.
      (Chunk 3 done: `cells` -> `materials: IdVec<BVoxPalette, IdRemap<
      BVoxMaterial, u32>>`; added `value_pools: IdRemap<BVoxValuePool, u32>`.)
- [x] Rebuild fixtures and the gc, validate, and remap tests; delete the
      duplicate-palette-ref test.
      (Chunk 3 done: all `voxcore` tests rebuilt to the binding/material/pool
      shape; the duplicate-palette-ref test is deleted and replaced with a
      two-layers-sharing-a-palette acceptance test; added pool-content rejection
      cases (empty, unordered bounds, non-integer int bound, value out of bounds,
      sRGB component out of range, negative linear component), an HDR-linear
      acceptance case, dangling-binding-pool and value-index-out-of-range cases,
      and the Phase 3 gate test. 53 tests pass, 52 under Miri.)

Gate: `voxcore` builds; build a two-layer object sharing a palette, validate it,
gc it, and read a material's resolved values back.

## Phase 4: `voxsmith` voxj seam

- [x] Rewrite `vox_palette_from_voxj_palette` to read column-major materials
      into the voxcore pool model and reject duplicate attributes rather than
      last-wins dedup. (Palette conversion now handles only bindings and the
      column-major-to-row material transpose; wire-color-kind canonicalization
      moved to the new per-pool `vox_value_pool_from_voxj_value_pool`, since
      pools are shared runtime-state entities, not per-palette. Recorded in the
      decisions log.)
- [x] Rewrite `voxj_palette_from_vox_palette` and `write_voxj` to build wire
      value pools from voxcore pools, emit column-major materials and bindings,
      and apply the `--color-format` choice. (Under the full-pool voxcore model
      the seam is a faithful 1:1 mirror in id order; document-wide `(kind,
      bounds)` dedup, the per-attribute bounds table, and material dedup are
      converter concerns for phases 5/6, because voxcore pools already carry
      bounds and are already the distinct set. `ColorFormat` (hex/float, default
      float) added and threaded through `write_voxj` and `VoxjFileBuilder`;
      the `--color-format` CLI flag lands in phase 7. Recorded in the log.)
- [x] Rework value decode/encode into kind-directed decode and encode, mapping
      color hex and int and float to and from the canonical float form. (Landed
      as the new pool-level converters `vox_value_pool_from_voxj_value_pool` and
      `voxj_value_pool_from_vox_value_pool`, one arm per kind. The named
      `vox_value_from_voxj_value`/`voxj_value_from_vox_value` stay unchanged and
      back only the `json` pool kind and `ext`, mirroring how voxcore's
      `VoxValue` and the wire's `VoxjValue` now back only json and ext. Per-value
      bound/range validation stays voxcore's `validate` job. Recorded in the
      log.)
- [x] Update the decoded-object seam
      (`vox_object_from_voxj_decoded_object`, `voxj_decoded_object_from_vox_object`)
      for layer refs and material-index samples, relying on voxcore now allowing
      duplicate palette refs. (`add_layer`/`voxel_material`/`iter_layers` and
      `BVoxMaterial` samples; two layers may share a palette.)
- [x] Point callers at the retargeted material-count helper.
      (`voxj_palette_material_counts` keyed by `layer_palette_refs` in both
      `from_voxj_file` and `write_voxj`.)
- [x] Rebuild the `from_voxj_file` fixtures; add two-layers-one-palette and
      min/max-violation cases and a `voxj -> vox -> voxj` round-trip that
      accounts for color canonicalization and the float default. (All fixtures
      rebuilt to the pool/binding/material/layer shape; the "tight" object now
      shares one palette across two layers; added `rejects_value_outside_pool_
      bounds`, `hex_color_canonicalizes_to_float_by_default`, and
      `hex_color_format_round_trips_hex`. `vox_palette_from_voxj_palette` gains
      transpose/dup-attribute/column-arity/ragged-column unit tests.)

Gate: a new-shape `.voxj` loads into `VoxMain` and writes back, stable under the
color and pool normalization. Verified: the crate cannot fully build until
phases 5-7 port the remaining voxcore consumers, so the 25 seam tests were built
and run against a `--no-default-features --features voxj` build (with the
old-API `reduce_palette` module temporarily gated out, then reverted); all pass.
`cargo check -p voxsmith` on default features leaves zero errors in the seam
files, only in the phase-5/6 modules.

## Phase 5: `voxsmith` color helpers and glTF pipeline

- [x] Move the shared attribute-name constants into one module set to the glTF
      names, imported by every converter, the glTF pipeline, and vxl. (Landed as
      the crate-root, always-compiled, public `gltf_attributes` module, so the
      non-gltf converters and vxl import it too; see the decisions log.)
- [x] Retarget `object_color_ref` to `baseColorFactor`, and generalize
      `cell_color` and `parse_color_hex` to resolve a color through
      material-to-binding-to-pool and decode by pool kind, not hex only.
      (Landed with the Phase 6 goxl chunk, the first `_color` consumer.
      `object_color_ref` now returns `(layer, palette, binding)` for the first
      layer binding `baseColorFactor`; `cell_color` takes `state` and resolves
      through `voxel_material` and `material_value`; `parse_color_hex` was renamed
      to `pool_color`, decoding a resolved `(pool, index)` by kind, sRGB straight
      to bytes and linear re-encoded. See the decisions log.)
- [x] Rename `MeshMaterial` fields and `MATERIAL_ATTRIBUTES` to the glTF vocab,
      split `emissive` into `emissiveFactor` color and `emissiveStrength` number,
      and set the default-scalar table to the new names with `metallicFactor`
      default 1. Keep writing `metallicFactor` explicitly where voxelize does so
      the matte look is preserved; the flip only bites absent-attribute defaults.
      (Build-path chunk done: `MeshMaterial` now carries `base_color`, `metallic`,
      `roughness`, `emissive_factor`, `emissive_strength`, and `occlusion` in the
      glTF vocab; `MATERIAL_ATTRIBUTES` and `cell_values`/`hex` are gone, the
      six-binding palette build moved into `voxelize_mesh`, which writes every
      scalar explicitly. The `default_scalar` table was already retargeted in the
      read/bake chunk.)
- [x] Redefine a material for the atlas as a material index into the selected
      layer (default 0) in `used_materials`; remove the cross-reference merge.
      (`resolve_used_materials` reads one layer by id and returns
      `Option<UsedMaterials>`; the merge is gone. The layer is a passed-in id, not
      a hardcoded 0, and the option is hoisted to the callers. See the log.)
- [x] Carry emissive as color plus strength through `sample_material`,
      `mesh_emissive_map`, and the `EmissiveColor` bake
      (`emissiveFactor * emissiveStrength` in linear); keep import and export in
      sync for glTF round-trips. (Build-path chunk done: the sampler now overrides
      the emissive COLOR per texel and leaves `emissiveStrength` as the material's
      flat scalar; `mesh_emissive_map` returns the linear emissive color instead
      of collapsing to the strongest channel. The bake side landed in the
      read/bake chunk.)
- [x] Update the glTF import (`from_gltf_bytes`) and its tests, and the atlas and
      material-document tests, for the renamed attributes and the color default.
      (Build-path chunk done: `from_gltf_bytes` reads `baseColorFactor` and the
      split `emissiveFactor`/`emissiveStrength`, and its tests read voxel values
      through the layer/material/binding/pool API. The atlas and material-document
      tests landed in the read/bake chunk.)

Chunk split (see decisions log): the read/bake chunk ported the glTF
material-atlas read/bake path, checklist items 1 and 4 plus the bake-side of items
3, 5, and 6. This build-path chunk finishes items 3, 5, and 6: `MeshMaterial`,
`sample_material`, `mesh_emissive_map`, `voxelize_mesh`, and the `from_gltf_bytes`
importer. The color helpers (item 2) remain deferred to Phase 6, where their only
callers are ported.

Gate: glTF export and import build and round-trip; a voxelized flat mesh renders
as before.

## Phase 6: `voxsmith` goxl, mvox, qbcl, vmax converters

First chunk (see decisions log): the unconditionally-compiled `reduce_palette`
straggler was ported to the pool/material model ahead of the listed items, since
it blocks every scoped converter build and phases 4 and 5 only worked around it by
temporarily cfg-gating it out. It clusters by `baseColorFactor` read through the
value-pool API, merges via `VoxMain::remove_material`, and dithers across layers;
its in-file tests were rebuilt to pools plus bindings plus materials, and both
known-pattern dither tests still pass byte-for-byte. Verified under
`--no-default-features --features voxj`. This is not one of the listed items below;
those stay unchecked.

- [x] goxl: build a color pool bound to `baseColorFactor`, emit materials as
      value-indices, and read color back through the pool-kind helper both in the
      direct and synthesized paths. (`from_goxl_file` builds one shared `srgba`
      pool of the distinct block colors bound to `baseColorFactor`, one material
      per color, and each object references it on one layer; `to_goxl_file` reads
      each voxel's color through `object_color_ref` plus `cell_color` in both the
      ext-driven `block_from_object` path and the synthesized `emit_object` path.
      The Phase 5 item 2 color helpers landed here too. Verified under
      `--no-default-features --features goxl`; 15 tests pass, the byte-exact
      round-trips included.)
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
