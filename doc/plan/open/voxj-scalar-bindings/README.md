# voxj scalar bindings: palette-scoped values and scalar layers

Status: **open.** Nothing has landed; this document records a design settled
with the owner, minus the [open questions](#open-questions). Phase 1 iterates
[reference/format-design.md](reference/format-design.md), the complete target
spec text, until the owner approves it; the executable steps live in
[checklist.md](checklist.md).

## Motivation

voxj can share one value across all materials of one palette today, but only
by contortion: a one-value pool bound by an all-zeros column. The constancy is
unvalidated content rather than structure, the intent is invisible, and no
other scope is reachable at all: two objects referencing the same palette
cannot carry different emissive strengths without cloning the palette.

Scalar bindings fix both. A palette may bind an attribute directly to a single
value-pool cell, alongside today's per-material columns, and an object gains a
second, channel-less layer list for referencing scalar-only palettes. The
motivating attribute is glTF's `emissiveStrength`; the mechanism is
kind-agnostic and works for any attribute and any pool kind.

## The format delta

A delta against
[voxel-json-file-format.md](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md);
the full replacement text is in
[reference/format-design.md](reference/format-design.md).

### Palette

`bindings` is renamed `arrayBindings`; its shape (`{ attribute, poolRef }`,
one `materials` column per entry) is unchanged. A new required sibling
`scalarBindings` holds `{ attribute, poolRef, valueRef }` entries, each
pinning an attribute to the single value `valuePools[poolRef].values[valueRef]`
for the whole palette; scalar bindings have no `materials` column. The
`materials` rules are untouched: columns are 1:1 with `arrayBindings` in
order; when `arrayBindings` is empty, `materials` is one empty array per
material and `M = materials.length`, so `materials: []` means `M = 0`.

A **scalar palette** is a palette with `arrayBindings: []` and
`materials: []`.

### Object

`layerPaletteRefs` is replaced by two required arrays of palette indices:
`arrayLayers` and `scalarLayers`. `voxelSamples` carries exactly one channel
per `arrayLayers` entry, in order. Scalar layers carry no channels and no
per-voxel data of any kind, and must reference scalar palettes.

### Resolution

1. Array layer `c`: read the voxel's sample `m`; each array binding takes
   `pool.values[materials[b][m]]`; each scalar binding on the same palette
   takes `pool.values[valueRef]`. The layer's contribution is the sampled row
   plus the palette's scalars.
2. Scalar layer: the contribution is its palette's scalar-binding values. No
   sample, no channel, no per-voxel read.
3. Unbound attributes take the vocabulary default. Cross-layer combination
   stays app-defined, exactly as today.

### Semantics

Scalar bindings are fixed values, not defaults and not modifiers. The format
defines wiring, never arithmetic; modifier behavior (`emissiveStrength`
multiplying `emissiveFactor`) already lives in the attribute vocabulary and
needs nothing from the mechanism. Within one palette an attribute may appear
in `arrayBindings` or `scalarBindings`, never both. "Scalar" means
single-valued, not numeric: a scalar binding may reference a color or `json`
cell.

### Validation deltas

All in existing rule shapes:

1. Palette closed over `{ arrayBindings, scalarBindings, materials }`; scalar
   binding closed over exactly `{ attribute, poolRef, valueRef }`; rules
   10.3/10.4 reworded from `bindings` to `arrayBindings`, content otherwise
   untouched, including `materials: []` meaning `M = 0`.
2. Rule 10.2 extends: no duplicate attribute across `arrayBindings` union
   `scalarBindings` of one palette.
3. `scalarBindings[].poolRef` indexes `valuePools`; `valueRef` is an integer
   in `[0, pool.values.length)`, the same check as a materials cell.
4. Object closed over the new keys; `arrayLayers` and `scalarLayers` both
   required arrays of palette indices, in range; rule 11 rewords to one
   channel per `arrayLayers` entry. No `M = 0` special case anywhere.
5. A `scalarLayers` entry must reference a scalar palette
   (`arrayBindings: []`, `materials: []`); anything else is dead data for that
   reference, and the format rejects rather than ignores.
6. No duplicate entries within one object's `scalarLayers` (rule-7 style): two
   identical scalar refs carry zero distinguishing information. Duplicates in
   `arrayLayers` stay legal; their channels distinguish them.

Unknown attribute names in `scalarBindings` are ignored, like `arrayBindings`
(advisory vocabulary). A scalar palette with `scalarBindings: []` is legal;
the format does not police pointlessness. Cross-layer attribute overlap (two
scalar layers supplying the same attribute, or a scalar layer overlapping an
array layer's binding) is not validated: it is app-defined meaning, the same
posture as two `baseColorFactor` array layers today.

## Usage idioms

The spec documents these in a Sharing Idioms subsection:

1. All materials of a palette share a value: `scalarBindings` on that palette,
   next to its array bindings. No extra layer needed.
2. Per-object variation over a shared palette: small scalar palettes
   referenced through `scalarLayers`; switching an object's knob is one
   integer.
3. Single source of truth: the pool cell; editing it updates every referencing
   palette.
4. Per-voxel escape hatch: move the attribute from `scalarBindings` to
   `arrayBindings` with a real column and channel.

## Decisions

Settled with the owner; do not reopen. The rejected alternatives behind them
are in [reference/design-notes.md](reference/design-notes.md).

1. **Single supply path.** Values reach materials only through palettes, and
   objects only reference palettes. No second reference form; a layer-attached
   constant ref was rejected for exactly this.
2. **Symmetry names the arity.** `arrayBindings` / `arrayLayers` carry
   per-voxel sampled data; `scalarBindings` / `scalarLayers` carry
   palette-scoped values with no per-voxel data. The names state the binding's
   arity (subject to open question 2 on the words themselves).
3. **No vestigial data.** Scalar layers have no channels, so geometry edits
   never touch scalar machinery and nothing information-free is stored. This
   is why the two-list split beat an `M = 0` carve-out in rule 11.
4. **Structure over convention.** Everything is a named structural fact a
   validator checks; constancy is not a convention buried in column content.
5. **Fixed values, not modifiers.** The format wires attributes to values;
   any modifier arithmetic lives in the attribute vocabulary.
6. **Reject dead data.** A non-scalar palette in `scalarLayers` and duplicate
   `scalarLayers` entries reject; both carry zero information for that
   reference.

## Open questions

Resolve with the owner in phase 1; each is marked `[OPEN n]` in
[reference/format-design.md](reference/format-design.md) where its wording
lands.

1. **Canonical layer order, back-to-front.** Cross-layer meaning stays
   app-defined, but an upcoming consumer renders layers back-to-front, and the
   owner is inclined to declare back-to-front the canonical `arrayLayers`
   order (first entry rearmost, last frontmost). Decide: normative spec
   wording or documented convention, like the glTF attribute vocabulary;
   whether `scalarLayers` order means anything (they carry no per-voxel data,
   so likely "no defined meaning"); and note that the two-list split
   forecloses interleaving scalar layers among array layers. Confirm that
   loss is acceptable.
2. **Naming.** `arrayBindings` / `scalarBindings` / `arrayLayers` /
   `scalarLayers` are the owner's working names. "Scalar" means single-valued,
   not numeric, and the spec says so. `columnBindings` / `valueBindings` are
   the noted alternates if that reading grates.
3. **`M = 0` palettes in `arrayLayers`.** Today's rules make an `M = 0`
   palette referenceable only by an empty object's layer (samples must lie in
   `[0, M)`), which stays true under the reworded rule 11. Keep that vacuous
   legality or reject scalar palettes in `arrayLayers` explicitly.
4. **Version.** Assume no compatibility machinery: the format changes in place
   and `version` stays `1`. Confirm cheaply.

## Milestones

1. **Format design.** Iterate
   [reference/format-design.md](reference/format-design.md) with the owner
   until approved, resolving the open questions. Nothing else starts first.
   The format gets perfect here, not in the spec commit.
2. **Spec.** Rewrite the format doc to the approved design in one commit, no
   code changes. From then on the spec is authoritative for any detail the
   plan leaves implicit.
3. **Code.** Phased by crate in dependency order (`voxj`, `voxj-codec`,
   `voxcore`, `voxsmith`, `vxl`, docs), fixtures regenerated in the same
   change that breaks them. End state: build, clippy, and tests green; crate
   READMEs and command docs consistent with the new spec.

## Blast radius

Surveyed 2026-07-14 by grepping `projects/` for `layerPaletteRefs` /
`layer_palette_refs`, `bindings`, `poolRef` / `pool_ref`, and `valuePools` /
`value_pools`. Only `voxsmith` and `vxl` consume `voxcore`; no tyt-side crate
does. There are no on-disk `.voxj` fixtures; every fixture is inline in tests
and regenerates with the code that breaks it.

| where | touched by |
| --- | --- |
| `voxj` | `VoxjPalette` (rename `bindings`, add scalar bindings), a new scalar-binding type beside `VoxjPaletteBinding`, `VoxjObject.layer_palette_refs` split into the two layer lists, doc comments |
| `voxj-codec` | `voxj_palette_material_counts`, `VoxjDecodedObject`, `encode_voxj_object` / `encode_voxj_object_optimized` / `decode_voxj_object` channel arity, `sample_encoding` docs, `check_voxj_file` / `validate_voxj_file` and `internal/voxj_validation/*` (reworded rules plus the new scalar checks), inline fixtures |
| `voxcore` | `VoxPalette` / `VoxPaletteBinding` (scalar-binding storage), `VoxObject` (scalar layer list), `vox_main` validate, `vox_gc_remap` / liveness (a pool referenced only by a scalar binding stays live), `vox_runtime_state`, in-test fixtures |
| `voxsmith` | the voxj seam (`internal/voxj/*` palette and object conversions, `write_voxj`), `convert/voxj` (`from_voxj_file`, `voxj_file_builder`, the `to_voxj*` paths), `reduce_palette`, material sampling (`internal/mesh/sample_material`, `mesh_material_maps`), and every converter that builds voxcore palettes (`gltf`, `vmax`, `voxelize`, goxl / mvox / qbcl) compiling against the renamed API |
| `vxl` | `mesh` (`attribute_binding`, `channel_source`, `channel_packing`, `texture_map`), `info` and `hierarchy show` (layer counts are now two lists), `palette show` / `palette list` (binding display), `validate` (new check names), `utilities/voxj_sample_encoding` |
| docs | the format spec; `voxj-codec/README.md`; the vxl-commands plan pages that state the palette and layer model (`README.md`, `reference/mesh.md`, `reference/palette/remap.md`, `reference/validate.md`); crate READMEs re-checked for stale wording |

Whether glTF import/export wires `emissiveStrength` through scalar bindings,
and whether other converters emit them, is scoped during phase 3 refinement
and logged in
[reference/implementation-decisions.md](reference/implementation-decisions.md);
the format change alone keeps converters at column parity.

## Documents

- [Implementation checklist](checklist.md): the phased task list. Start here
  when executing.
- [Format design](reference/format-design.md): the complete target spec text,
  the phase 1 iteration surface. Any format question is settled here, then by
  the spec once phase 2 lands.
- [Design notes](reference/design-notes.md): rejected alternatives and strain
  noticed while drafting.
- [Implementation decisions](reference/implementation-decisions.md):
  code-level decisions recorded during execution.
- [Session prompt](continue-voxj-scalar-bindings.md): how to resume this work
  in a fresh session.
