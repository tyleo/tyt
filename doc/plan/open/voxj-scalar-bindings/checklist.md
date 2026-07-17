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

- [ ] Owner reads the draft; fold in wording and structure edits until the
      text stands on its own.
- [x] Resolve open question 1, override order: layers combine by overriding
      in `layers` order, back to front; each attribute takes its value from
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
- [x] Fold each resolution into format-design.md (drop its `[OPEN n]`
      marker) and move it to the README's decisions with rationale.

Gate: the owner approves format-design.md.

## Phase 2: spec

- [ ] Rewrite
      `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md` to the
      approved design in one commit. No code changes. Every touched section:
      Objects, the Sample Encodings intro and encoding items and example
      title, Voxel Order, the Value Pools intro cross-reference, Palettes
      plus the new Sharing Idioms subsection, Attributes, Validation rules
      5, 6, 8, 10, and 11, Versioning item 4, the File Example, and the
      TypeScript schema.
- [ ] Sweep the whole spec for stale `bindings` / `layerPaletteRefs` /
      per-layer wording the section list missed.

Gate: the spec matches format-design.md; the reference file stops being
authoritative.

## Phase 3: `voxj` (coarse)

- [ ] Rewrite `VoxjPalette` to array plus scalar bindings; add the
      scalar-binding type; rename `VoxjObject.layer_palette_refs` to the
      ordered `layers` list; keep `deny_unknown_fields` closure; update doc
      comments to the new framing; in-crate round-trip coverage.

## Phase 4: `voxj-codec` (coarse)

- [ ] Re-derive the material-count and channel-arity paths from sampled
      layers (palette `M > 0`); rename `VoxjDecodedObject` fields; rework the
      validation rules (closures, 10.2 union uniqueness, `valueRef` range,
      the sampled-layer channel rule) as named checks; regenerate inline
      fixtures.

## Phase 5: `voxcore` (coarse)

- [ ] Grow `VoxPalette` scalar-binding storage; derive sampled layers for
      channel arity; extend `vox_main` validate, gc, remap, and liveness (a
      pool referenced only by a scalar binding stays live); regenerate
      in-test fixtures.

## Phase 6: `voxsmith` (coarse)

- [ ] Rework the voxj seam (palette and object conversions, `write_voxj`,
      `voxj_file_builder`) for the new shapes; extend `reduce_palette` and
      material sampling over scalar contributions and the canonical
      layer-override order; keep every converter
      compiling at column parity; decide and log whether glTF wires
      `emissiveStrength` through scalar bindings; regenerate fixtures.

## Phase 7: `vxl` and docs (coarse)

- [ ] Surface the layer list, sampled vs unsampled, and scalar bindings in
      `info`, `hierarchy show`, `palette show` / `list`, and `mesh`; add the
      new `validate` check names; regenerate goldens.
- [ ] Bring `voxj-codec/README.md`, the other crate READMEs, and the
      vxl-commands plan pages (`README.md`, `reference/mesh.md`,
      `reference/palette/remap.md`, `reference/validate.md`) in line with the
      new spec.

Gate: build, clippy, and tests green across the workspace; docs consistent.
Close the plan.
