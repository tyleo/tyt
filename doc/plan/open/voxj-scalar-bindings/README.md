# voxj scalar bindings: palette-scoped values and scalar layers

Status: **open.** This document records a design settled with the owner, the
former [open questions](#open-questions) included. The owner approved
[reference/format-design.md](reference/format-design.md), the complete target
spec text, on 2026-07-16, closing phase 1, and the phase 2 spec commit landed
2026-07-17, so the
[spec](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md)
is now authoritative; the next step is phase 3, the `voxj` crate. The
executable steps live in [checklist.md](checklist.md).

## Motivation

voxj can share one value across all materials of one palette today, but only
by contortion: a one-value pool bound by an all-zeros column. The constancy is
unvalidated content rather than structure, the intent is invisible, and no
other scope is reachable at all: two objects referencing the same palette
cannot carry different emissive strengths without cloning the palette.

Scalar bindings fix both. A palette may bind a property directly to a single
value-pool cell, alongside today's per-material columns, and a palette with no
materials is never sampled, so an object layer can supply palette-scoped
values to the whole object with no per-voxel channel. The motivating property
is glTF's `emissiveStrength`; the mechanism is kind-agnostic and works for any
property and any pool kind.

## The format delta

A delta against
[voxel-json-file-format.md](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md);
the full replacement text is in
[reference/format-design.md](reference/format-design.md).

### Palette

`bindings` is renamed `arrayProperties`; its shape (`{ name, valuePool }`,
one `materials` column per entry) is otherwise unchanged: the `attribute`
field is renamed `name` and `poolRef` is renamed `valuePool` (decisions 11
and 12). A new required sibling `scalarProperties` holds
`{ name, valuePool, valueIndex }` entries, each pinning a property to the
single value `valuePools[valuePool].values[valueIndex]` for the whole
palette; scalar properties have no `materials` column. A palette may carry
array properties, scalar properties, both, or neither. `materials` is row-major
(decision 14): one row per material, a value-index per array property in
property order, so `M = materials.length` and `materials: []` means
`M = 0`.

### Object

`layerPaletteRefs` is renamed `layers`: still one array of palette indices,
now ordered back to front, repeats included. Each layer supplies all
of its palette's properties: scalar properties one value for the whole
object, array properties one value per voxel. A layer is sampled iff its palette's
material count `M > 0`; `voxelSamples` carries exactly one channel per sampled
layer, in `layers` order, so a scalar-only palette (`materials: []`) carries
no channel.

### Hierarchy

`hierarchyNodes` is renamed `nodes` and `rootHierarchyNodes` is renamed
`rootNodes` (decision 13); the node shape and `childNodes` / `childObjects`
are untouched.

### Resolution

1. Each layer supplies its palette's properties: a scalar property supplies
   its name from `pool.values[valueIndex]`, one value for the whole object;
   an array property reads the voxel's sample `m` from the layer's channel
   and takes `pool.values[materials[m][b]]`. An unsampled layer supplies
   only its scalar properties.
2. Layers override canonically: contributions apply in `layers` order, back
   to front, and each property takes its value from the last layer that
   supplies it. The author orders overrides freely: a scalar layer listed
   after an array layer replaces per-voxel values with an object-wide one.
3. Unbound properties are left to the vocabulary; the recommended glTF
   conventions supply a default for each.

### Semantics

A scalar property wires a name to a value; any arithmetic, such as
`emissiveStrength` multiplying `emissiveFactor`, comes from the property
vocabulary. Within one palette a name may appear in `arrayProperties` or
`scalarProperties`, so a single layer never conflicts with itself. "Scalar" means single-valued: a scalar property may reference a cell
of any pool kind.

### Validation deltas

All in existing rule shapes:

1. Palette closed over `{ arrayProperties, scalarProperties, materials }`;
   scalar property closed over exactly `{ name, valuePool, valueIndex }`;
   rule 10.3 is the row rule: `materials` is `M >= 0` rows of exactly
   `arrayProperties.length` value-indices, so `materials: []` means `M = 0`
   (decision 14).
2. Rule 10.2 extends: no duplicate `name` across `arrayProperties` union
   `scalarProperties` of one palette.
3. `scalarProperties[].valuePool` indexes `valuePools`; `valueIndex` is an
   integer in `[0, pool.values.length)`, the same check as a materials cell.
4. Object closed over the new key; `layers` a required array of palette
   indices, in range; rule 11 rewords to one channel per sampled layer, where
   a layer is sampled iff its palette's `M > 0`. An `M = 0` palette is never
   sampled, so the channel rules need no `M = 0` case.

Unknown property names in `scalarProperties` are ignored, like
`arrayProperties` (advisory vocabulary). Neither layer overlap nor repeated layer references
are validated: the override order gives both their meaning, and the format
does not police pointlessness (an empty `scalarProperties` or a fully
shadowed layer is legal).

## Usage idioms

The spec documents these in a Sharing Idioms subsection:

1. All materials of a palette share a value: `scalarProperties` on that
   palette; one `layers` entry supplies both arities.
2. Per-object variation over a shared palette: small one-scalar-property
   palettes with no materials, listed after the shared palette; switching an
   object's knob is one integer.
3. Single source of truth: the pool cell; editing it updates every referencing
   palette.
4. Per-voxel escape hatch: move the property from `scalarProperties` to
   `arrayProperties` with a real column and channel.
5. Whole-object override: a scalar-property palette listed after an array
   layer replaces its per-voxel values for that property.

## Decisions

Settled with the owner; do not reopen. The rejected alternatives behind them
are in [reference/design-notes.md](reference/design-notes.md).

1. **Single supply path.** Values reach materials only through palettes, and
   objects only reference palettes. No second reference form; a layer-attached
   constant ref was rejected for exactly this.
2. **Symmetry names the arity.** `arrayProperties` / `scalarProperties`
   carry per-voxel sampled data and palette-scoped values respectively; the
   names state the entry's arity (closed open question 2 as `arrayBindings`
   / `scalarBindings`, owner review 2026-07-15; renamed by decision 12). The
   object side is the single `layers` list (decision 10).
3. **No vestigial data.** A layer whose palette has no materials is never
   sampled and carries no channel, so geometry edits never touch scalar
   machinery and nothing information-free is stored. The sampled-iff-`M > 0`
   rule supplies this structurally (owner review 2026-07-15; previously via
   the two-list split).
4. **Structure over convention.** Everything is a named structural fact a
   validator checks; constancy is not a convention buried in column content.
5. **Fixed values, not modifiers.** The format wires properties to values;
   any modifier arithmetic lives in the property vocabulary.
6. **Just palettes.** There is no palette classification: a palette may carry
   array properties, scalar properties, both, or neither, and `layers` may
   reference any palette, repeats included. A layer supplies all of its
   palette's properties. Replaces the earlier scalar-palette shape rule and,
   with decision 10, the one-arity-per-list rule (owner reviews, 2026-07-15).
7. **Canonical override.** Layers combine by overriding, not app-defined
   merge: contributions apply in `layers` order, back to front, and each
   property takes its value from the last layer that supplies it. The author
   orders overrides, so a scalar layer listed after an array layer replaces
   per-voxel values with an object-wide one (owner review 2026-07-15; revised
   from the fixed scalars-then-arrays order that first closed open
   question 1).
8. **`M = 0` means unsampled.** A palette with no materials is never sampled:
   a layer referencing it carries no channel and supplies only scalar
   properties. Replaces the earlier vacuous-legality posture, moot now that
   channel count derives from palette shape (owner review 2026-07-15; revised
   from the first closure of open question 3).
9. **Version stays `1`.** The format changes in place, with no compatibility
   machinery (owner review 2026-07-15; closed open question 4).
10. **One layer list, channels derived.** `arrayLayers` / `scalarLayers`
    merged into a single ordered `layers` list; `voxelSamples` has one
    channel per sampled layer rather than a stated count. Mixed palettes are
    listed once, and the derived rule costs only the palette dereference
    rule 11 already does (owner review 2026-07-15).
11. **Properties, not attributes.** The concept is named property, and the
    spec's Attributes section retitles to Properties. glTF, the recommended
    vocabulary, reserves attribute for per-vertex mesh data and calls
    material parameters properties, so `attribute` invited collision
    anywhere materials and meshes are discussed together (owner review
    2026-07-16; the entry field, first named `property`, became `name` with
    decision 12).
12. **Plain-named fields.** The palette lists are `arrayProperties` /
    `scalarProperties`, their entries `{ name, valuePool }` and
    `{ name, valuePool, valueIndex }`. An entry is the property itself, so
    its key field is `name` and the binding noun drops as a second concept.
    Reference fields name what they point at with no `Ref` marker, like
    `layers`, `childNodes`, and glTF's `"mesh": 0`. Bare `value` was
    rejected: over a numeric pool an index reads as the bound value itself,
    while `valueIndex` matches the "value-index" the materials rules already
    use (owner review 2026-07-17; revises the `arrayBindings` /
    `scalarBindings` and `property` / `poolRef` / `valueRef` names of
    decisions 2 and 11).
13. **One name for nodes.** The wire fields are `nodes` and `rootNodes`,
    renamed from `hierarchyNodes` and `rootHierarchyNodes`. voxj has one
    node kind, `childNodes` already used the short name, and glTF calls the
    array `nodes`, so the long forms named the same entity two ways. The
    prose term hierarchy node and the TS `HierarchyNode` interface stay
    (owner review 2026-07-17).
14. **Rows are materials.** `materials` is row-major: one row per material,
    a value-index per array property, so `M = materials.length` and each
    entry is a material. voxcore already stores materials per-material and
    every converter and reader works row-wise, so the wire's columns bought
    only a transposing seam; measured under whole-file deflate the two
    orientations are within bytes of each other, rows smaller at large `M`
    (owner review 2026-07-17).

## Open questions

None. All four closed in owner review 2026-07-15; a second review the same
day merged the layer lists into one (decision 10), revising decisions 3 and
6 to 8, a review 2026-07-16 renamed attribute to property (decision 11), and
a review 2026-07-17 renamed the palette lists and fields to
`arrayProperties` / `scalarProperties` and `name` / `valuePool` /
`valueIndex` (decision 12) and the hierarchy fields to `nodes` /
`rootNodes` (decision 13), and made `materials` row-major (decision 14).

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
| `vxl` | `mesh` (`attribute_binding`, `channel_source`, `channel_packing`, `texture_map`, the `--define-attribute` flag), `info` and `hierarchy show` (sampled vs unsampled layer counts), `palette show` / `palette list` (binding display, `attribute_ref` / `attribute_selector`), `validate` (new check names), `implementation/attribute_names`, `utilities/voxj_sample_encoding`; the attribute-to-property rename touches every `attribute`-named identifier and flag here |
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
