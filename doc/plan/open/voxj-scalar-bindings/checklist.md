# voxj scalar bindings implementation checklist

Tracks the format change and the migration to it. Read the
[README](README.md) for the delta, the decisions, and the open questions;
[reference/format-design.md](reference/format-design.md) is the target spec
text. Log code-level decisions in
[reference/implementation-decisions.md](reference/implementation-decisions.md).
Check items off as they land.

## Ground rules

- One reviewable chunk per step. `cargo fmt --all`, `cargo clippy --workspace
  --all-targets -- -D warnings`, and `cargo test` before each commit; the
  pre-commit hook enforces fmt + clippy.
- Phase order is strict: the format design is approved before the spec commit,
  and the spec lands before any code moves. Once it lands, the spec is
  authoritative for any detail the plan leaves implicit.
- Fixtures are regenerated in the same change that breaks them; every fixture
  is inline in tests, none on disk.
- Phases 3 to 7 are deliberately coarse. Refine them into commit-sized items
  once the spec lands, the way the voxj-redesign checklist was refined during
  execution, and expect the workspace to be red between crate phases.

## Phase 1: format design

Iterate [reference/format-design.md](reference/format-design.md) with the
owner until approved. Nothing else starts first; the format gets perfect
here, not in the spec commit.

- [x] Owner reads the draft; fold in wording and structure edits until the
      text stands on its own. (Approved 2026-07-16 with no edits.)
- [x] Resolve open question 1, override order: layers combine by overriding
      in `layers` order, back to front; each property takes its value from
      the last layer that supplies it. (First closed as `scalarLayers` then
      `arrayLayers`; revised by the layer-list merge below.)
- [x] Resolve open question 2, naming: `arrayBindings` / `scalarBindings`
      confirmed as final; the object side is the single `layers` list.
- [x] Resolve open question 3, `M = 0` palettes: an `M = 0` palette is never
      sampled, so the vacuous case is gone. (First closed as vacuous
      legality kept; revised by the layer-list merge below.)
- [x] Resolve open question 4, version: stays `1`, the format changes in
      place.
- [x] Owner review 2026-07-15, second pass: merge `arrayLayers` /
      `scalarLayers` into one ordered `layers` list; a layer is sampled iff
      its palette's `M > 0`, and only sampled layers carry `voxelSamples`
      channels (README decision 10).
- [x] Owner review 2026-07-16: rename `attribute` to `property` across the
      whole format; the bindings' field, the Attributes section retitled
      Properties, and the glTF and Value Pool Kinds table headers (README
      decision 11).
- [x] Owner review 2026-07-17, during the phase 2 review: rename to plain
      fields; `arrayBindings` / `scalarBindings` become `arrayProperties` /
      `scalarProperties`, and entry fields are `name` / `valuePool` /
      `valueIndex`, replacing `property` / `poolRef` / `valueRef` (README
      decision 12).
- [x] Owner review 2026-07-17, post-spec-commit: rename `hierarchyNodes` /
      `rootHierarchyNodes` to `nodes` / `rootNodes` (README decision 13),
      and drop the trailing validation note as restating the Objects
      section and rule 11.
- [x] Owner review 2026-07-17: `materials` goes row-major; one row per
      material, a value-index per array property in property order, and
      `M = materials.length` (README decision 14).
- [x] Fold each resolution into format-design.md (drop its `[OPEN n]`
      marker) and move it to the README's decisions with rationale.

Gate: the owner approves format-design.md. Passed 2026-07-16.

## Phase 2: spec

- [x] Rewrite
      `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md` to the
      approved design in one commit. No code changes. Every touched section:
      Objects, the Sample Encodings intro and encoding items and example
      title, Voxel Order, the Value Pools intro cross-reference and the
      Value Pool Kinds table header, Palettes plus the new Sharing Idioms
      subsection, Attributes (retitled Properties), Validation rules 5, 6,
      8, 10, and 11, Versioning item 4, the File Example, and the
      TypeScript schema.
- [x] Sweep the whole spec for stale `bindings` / `layerPaletteRefs` /
      `attribute` / per-layer wording the section list missed. (Also
      Versioning item 5 and the glTF conventions intro, color, and emission
      paragraphs.)

Gate: the spec matches format-design.md; the reference file stops being
authoritative. Passed 2026-07-17.

## Phase 3: `voxj`

Refined 2026-07-19: the crate is small enough that the whole phase is one
commit-sized chunk.

- [x] Rewrite `VoxjPalette` to array plus scalar properties with `name` /
      `value_pool` / `value_index` fields and row-major `materials`
      (`VoxjArrayProperty` and the new `VoxjScalarProperty` replace
      `VoxjPaletteBinding`); rename `VoxjObject.layer_palette_refs` to the
      ordered `layers` list and the hierarchy fields to `nodes` /
      `root_nodes`; keep `deny_unknown_fields` closure; update doc comments
      to the new framing; in-crate round-trip coverage.

## Phase 4: `voxj-codec`

Refined 2026-07-19 into two commits: the wire adaptation first, then the
scalar-property content checks on top of it.

- [x] Adapt the crate to the new wire model: rename `layers` / `nodes` /
      `root_nodes` throughout, derive `M = materials.len()` in
      `voxj_palette_material_counts`, carry one channel per sampled layer
      (`M > 0`) through `VoxjDecodedObject`, `encode_voxj_object` /
      `encode_voxj_object_optimized` / `decode_voxj_object`, and
      `check_geometry`; rewrite `check_palettes` to the array-property and
      row rules (10.1 array side, 10.3, 10.4); update check and codec doc
      comments; regenerate inline fixtures, adding sampled-vs-unsampled
      coverage. Scalar properties parse and round-trip but are not yet
      content-checked.
- [x] Add the scalar-property checks: non-empty names and rule 10.2 name
      uniqueness across `arrayProperties` union `scalarProperties`, scalar
      `valuePool` in range, `valueIndex` in `[0, pool.values.length)`
      (rule 10.5); extend the valid fixtures with scalar properties and add
      one failure fixture per new check; finalize the `palettes` check
      wording in `check_voxj_file` and `Check`.

## Phase 5: `voxcore`

Refined 2026-07-19 into three commits: the property-vocabulary rename first,
then scalar-property storage and checks, then sampled-layer derivation.

- [x] Rename the palette surface to the spec's property vocabulary, no
      behavior change: `VoxPaletteBinding` becomes `VoxArrayProperty`
      (`name` / `pool`, brand `BVoxArrayProperty`), the `VoxPalette` storage
      and API follow (`add_array_property`, `array_property_by_name`, and
      so on), materials are described as rows (README decision 14), and the
      `Error` variants become `ArrayPropertyPool` /
      `DuplicateArrayPropertyName` with `MaterialValue` gaining an
      `array_property` field. Owner review during the session added
      branded value ids: pools store `IdVec<BVoxPoolValue, T>` and every
      pool-value reference is a `U32Id<BVoxPoolValue>` named a value id
      (`value_id`, `remap_pool_value_ids`; the wire keeps `valueIndex`).
- [x] Add scalar properties: `VoxScalarProperty` (`name` / `pool` /
      `value_id`, brand `BVoxScalarProperty`), palette storage, name
      maps, and API; gc, clone, and removal integration;
      `relabel_value_pools` and `remap_pool_value_ids` cover scalar
      cells; `prune_value_pools` keeps a value alive that only a scalar
      property references; `reorder_value_pool` follows them; `validate`
      checks scalar pools, `value_id` range, and name uniqueness across
      both lists; a scalar resolution helper beside `material_value`.
- [x] Derive sampledness from the palette: a layer is sampled iff its
      palette's material count is above zero; scope the live-voxel sample
      checks in `validate` to sampled layers; expose the sampled-layer view
      the voxsmith seam and vxl need (`layer_is_sampled`,
      `iter_sampled_layers`, and the `VoxObject::layer_palette` accessor
      backing them); document unsampled layers' ignored sample cells and
      exempt them in object gc.

## Phase 6: `voxsmith`

Refined 2026-07-19 into three commits: the crate-wide adaptation and seam
rework first, then scalar-aware reduction and sampling, then the glTF
decision.

- [x] Adapt the crate to the renamed voxcore and voxj APIs and rework the
      voxj seam for the new shapes: row-major materials (both transposes
      drop), scalar properties carried through the palette conversions,
      one sample channel per sampled layer in the object conversions
      (`vox_object_from_voxj_decoded_object` takes material counts,
      `voxj_decoded_object_from_vox_object` reads sampledness off the
      `VoxMain`), and the `nodes` / `root_nodes` renames; every other
      converter compiles at column parity; regenerate fixtures, adding
      scalar-property and unsampled-layer round-trip coverage.
- [x] Extend `reduce_palette` and material sampling
      (`internal/mesh/sample_material`, `mesh_material_maps`) over scalar
      contributions and the canonical layer-override order. (The named
      mesh files sample glTF textures, not palettes; the extension landed
      in the shared `ObjectPropertyRef` resolution behind the color
      exporters, the atlas bake's scalar fallback, and reduction coverage
      for scalar-carrying palettes and unsampled layers. See the
      decisions log.)
- [x] Decide and log whether glTF import/export wires `emissiveStrength`
      through scalar properties; implement the decision. (Import pins a
      strength every distinct material shares as a scalar property; mixed
      strengths keep the column. Export already resolved both arities; the
      vmax material fold gained the scalar fallback so the pin survives
      glTF to vmax. See the decisions log.)

## Phase 7: `vxl` and docs

Refined 2026-07-20 into four commits: the compile adaptation first, then the
property rename, then the new-model surfaces, then the docs sweep.

- [x] Adapt the crate to the renamed voxcore API at display parity:
      `iter_array_properties` / `array_property_by_name` / `value_id` and
      branded pool-value ids through `palette_show`, `mesh_object`, and
      `attribute_names`, plus the test fixtures in `info`, `palette_list`,
      and `hierarchy_show`; scalar properties stay unsurfaced and every
      flag, header, and JSON key keeps its `attribute` wording. The
      workspace compiles and tests green again.
- [ ] Align the crate with the property rename: `attribute`-named files and
      identifiers (`attribute_names`, `attribute_ref`, `attribute_selector`,
      `attribute_binding`, `ChannelSource::Attribute`), the
      `--define-attribute` flag, and the displayed vocabulary (table
      headers, JSON keys, error messages); regenerate inline expectations.
- [ ] Surface the new model: scalar properties in `palette show` /
      `palette list` / `info` and the mesh channel lookup
      (`property_by_name`), sampled vs unsampled layer counts in `info` and
      `hierarchy show`; `validate` gains no new check names (the voxj-codec
      check surface is unchanged); regenerate inline expectations.
- [ ] Bring `voxj-codec/README.md`, the other crate READMEs, and the
      vxl-commands plan pages (`README.md`, `reference/mesh.md`,
      `reference/palette/remap.md`, `reference/validate.md`) in line with the
      new spec.

Gate: build, clippy, and tests green across the workspace; docs consistent.
Close the plan.
