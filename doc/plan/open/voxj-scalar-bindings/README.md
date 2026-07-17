# voxj scalar bindings: palette-scoped values and scalar layers

Status: **open.** Nothing has landed; this document records a design settled
with the owner, the former [open questions](#open-questions) included. Phase 1
iterates [reference/format-design.md](reference/format-design.md), the
complete target spec text, until the owner approves it; the executable steps
live in [checklist.md](checklist.md).

## Motivation

voxj can share one value across all materials of one palette today, but only
by contortion: a one-value pool bound by an all-zeros column. The constancy is
unvalidated content rather than structure, the intent is invisible, and no
other scope is reachable at all: two objects referencing the same palette
cannot carry different emissive strengths without cloning the palette.

Scalar bindings fix both. A palette may bind an attribute directly to a single
value-pool cell, alongside today's per-material columns, and a palette with no
materials is never sampled, so an object layer can supply palette-scoped
values to the whole object with no per-voxel channel. The motivating attribute
is glTF's `emissiveStrength`; the mechanism is kind-agnostic and works for any
attribute and any pool kind.

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
for the whole palette; scalar bindings have no `materials` column. A palette
may carry array bindings, scalar bindings, both, or neither. The `materials` rules are
untouched: columns are 1:1 with `arrayBindings` in order; when `arrayBindings`
is empty, `materials` is one empty array per material and
`M = materials.length`, so `materials: []` means `M = 0`.

### Object

`layerPaletteRefs` is renamed `layers`: still one array of palette indices,
now ordered back to front, repeats included. Each layer supplies all
of its palette's bindings: scalar bindings one value for the whole object,
array bindings one value per voxel. A layer is sampled iff its palette's
material count `M > 0`; `voxelSamples` carries exactly one channel per sampled
layer, in `layers` order, so a scalar-only palette (`materials: []`) carries
no channel.

### Resolution

1. Each layer supplies its palette's bindings: a scalar binding supplies its
   attribute from `pool.values[valueRef]`, one value for the whole object; an
   array binding reads the voxel's sample `m` from the layer's channel and
   takes `pool.values[materials[b][m]]`. An unsampled layer supplies only its
   scalar bindings.
2. Layers override canonically: contributions apply in `layers` order, back
   to front, and each attribute takes its value from the last layer that
   supplies it. The author orders overrides freely: a scalar layer listed
   after an array layer replaces per-voxel values with an object-wide one.
3. Unbound attributes take the vocabulary default.

### Semantics

A scalar binding wires an attribute to a value; any arithmetic, such as
`emissiveStrength` multiplying `emissiveFactor`, comes from the attribute
vocabulary. Within one palette an attribute may appear in `arrayBindings` or
`scalarBindings`, never both, so a single layer never conflicts with itself.
"Scalar" means single-valued: a scalar binding may reference a cell of any
pool kind.

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
4. Object closed over the new key; `layers` a required array of palette
   indices, in range; rule 11 rewords to one channel per sampled layer, where
   a layer is sampled iff its palette's `M > 0`. An `M = 0` palette is never
   sampled, so the channel rules need no `M = 0` case.

Unknown attribute names in `scalarBindings` are ignored, like `arrayBindings`
(advisory vocabulary). Neither layer overlap nor repeated layer references
are validated: the override order gives both their meaning, and the format
does not police pointlessness (an empty `scalarBindings` or a fully shadowed
layer is legal).

## Usage idioms

The spec documents these in a Sharing Idioms subsection:

1. All materials of a palette share a value: `scalarBindings` on that palette;
   one `layers` entry supplies both arities.
2. Per-object variation over a shared palette: small one-scalar-binding
   palettes with no materials, listed after the shared palette; switching an
   object's knob is one integer.
3. Single source of truth: the pool cell; editing it updates every referencing
   palette.
4. Per-voxel escape hatch: move the attribute from `scalarBindings` to
   `arrayBindings` with a real column and channel.
5. Whole-object override: a scalar-binding palette listed after an array
   layer replaces its per-voxel values for that attribute.

## Decisions

Settled with the owner; do not reopen. The rejected alternatives behind them
are in [reference/design-notes.md](reference/design-notes.md).

1. **Single supply path.** Values reach materials only through palettes, and
   objects only reference palettes. No second reference form; a layer-attached
   constant ref was rejected for exactly this.
2. **Symmetry names the arity.** `arrayBindings` / `scalarBindings` carry
   per-voxel sampled data and palette-scoped values respectively; the names
   state the binding's arity, confirmed as final in owner review 2026-07-15
   (closed open question 2). The object side is the single `layers` list
   (decision 10).
3. **No vestigial data.** A layer whose palette has no materials is never
   sampled and carries no channel, so geometry edits never touch scalar
   machinery and nothing information-free is stored. The sampled-iff-`M > 0`
   rule supplies this structurally (owner review 2026-07-15; previously via
   the two-list split).
4. **Structure over convention.** Everything is a named structural fact a
   validator checks; constancy is not a convention buried in column content.
5. **Fixed values, not modifiers.** The format wires attributes to values;
   any modifier arithmetic lives in the attribute vocabulary.
6. **Just palettes.** There is no palette classification: a palette may carry
   array bindings, scalar bindings, both, or neither, and `layers` may
   reference any palette, repeats included. A layer supplies all of its palette's bindings.
   Replaces the earlier scalar-palette shape rule and, with decision 10, the
   one-arity-per-list rule (owner reviews, 2026-07-15).
7. **Canonical override.** Layers combine by overriding, not app-defined
   merge: contributions apply in `layers` order, back to front, and each
   attribute takes its value from the last layer that supplies it. The author
   orders overrides, so a scalar layer listed after an array layer replaces
   per-voxel values with an object-wide one (owner review 2026-07-15; revised
   from the fixed scalars-then-arrays order that first closed open
   question 1).
8. **`M = 0` means unsampled.** A palette with no materials is never sampled:
   a layer referencing it carries no channel and supplies only scalar
   bindings. Replaces the earlier vacuous-legality posture, moot now that
   channel count derives from palette shape (owner review 2026-07-15; revised
   from the first closure of open question 3).
9. **Version stays `1`.** The format changes in place, with no compatibility
   machinery (owner review 2026-07-15; closed open question 4).
10. **One layer list, channels derived.** `arrayLayers` / `scalarLayers`
    merged into a single ordered `layers` list; `voxelSamples` has one
    channel per sampled layer rather than a stated count. Mixed palettes are
    listed once, and the derived rule costs only the palette dereference
    rule 11 already does (owner review 2026-07-15).

## Open questions

None. All four closed in owner review 2026-07-15; a second review the same
day merged the layer lists into one (decision 10), revising decisions 3 and
6 to 8.

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
| `voxj` | `VoxjPalette` (rename `bindings`, add scalar bindings), a new scalar-binding type beside `VoxjPaletteBinding`, `VoxjObject.layer_palette_refs` renamed to the ordered `layers` list, doc comments |
| `voxj-codec` | `voxj_palette_material_counts`, `VoxjDecodedObject`, `encode_voxj_object` / `encode_voxj_object_optimized` / `decode_voxj_object` channel arity (one channel per sampled layer, derived from palette material counts), `sample_encoding` docs, `check_voxj_file` / `validate_voxj_file` and `internal/voxj_validation/*` (reworded rules plus the new scalar checks), inline fixtures |
| `voxcore` | `VoxPalette` / `VoxPaletteBinding` (scalar-binding storage), `VoxObject` (sampled-layer channel derivation), `vox_main` validate, `vox_gc_remap` / liveness (a pool referenced only by a scalar binding stays live), `vox_runtime_state`, in-test fixtures |
| `voxsmith` | the voxj seam (`internal/voxj/*` palette and object conversions, `write_voxj`), `convert/voxj` (`from_voxj_file`, `voxj_file_builder`, the `to_voxj*` paths), `reduce_palette`, material sampling (`internal/mesh/sample_material`, `mesh_material_maps`), and every converter that builds voxcore palettes (`gltf`, `vmax`, `voxelize`, goxl / mvox / qbcl) compiling against the renamed API |
| `vxl` | `mesh` (`attribute_binding`, `channel_source`, `channel_packing`, `texture_map`), `info` and `hierarchy show` (sampled vs unsampled layer counts), `palette show` / `palette list` (binding display), `validate` (new check names), `utilities/voxj_sample_encoding` |
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
