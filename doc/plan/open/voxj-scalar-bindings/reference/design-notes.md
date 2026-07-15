# Scalar bindings design notes

_Part of the [voxj scalar-bindings plan](../README.md)._

Rejected alternatives and the strain noticed while drafting
[format-design.md](format-design.md). The settled rationale is in the
[README decisions](../README.md#decisions).

## Rejected alternatives

1. **Status quo: one-value pool plus an all-zeros column.** Constancy is
   unvalidated content, intent is invisible, and per-object variation over a
   shared palette forces palette cloning.
2. **Palette-level constants alone, no scalar layers.** Palette state aliases
   across every referencing layer, so per-object variation still forces
   cloning.
3. **Layer-attached constant refs**, `{ attribute, poolRef, valueRef }` on
   the layer. Works, but adds a second supply path for attribute values
   outside palettes; rejected to keep palettes the only attribute carrier.
4. **Knob-as-material with a `composes` palette flag**, a format-defined
   layer merge. Requires cross-layer merge semantics the format deliberately
   refuses to define, and carries the knob in channel content: a phantom
   `M = 1` material whose validator-forced uniform channel must track the
   voxel count.
5. **`M = 0` palettes with an empty-channel carve-out in rule 11.**
   Superseded by the two-list split, which removes the vestigial channel
   structurally instead of by rule exception.
6. **Hierarchy-node attachment** for per-instance variation. Makes material
   resolution path-dependent through the instancing DAG and demands
   inheritance rules; out of scope. Nothing in this design forecloses adding
   it later.
7. **Top-level named constants registry.** Scalar palettes plus their pools
   are already enumerable; a display name is an editor concern the wire
   format does not need.

## Strain noticed while drafting

1. **Scalar palette is not just "no array bindings".** `arrayBindings: []`
   with `materials: [[], []]` (`M = 2`, all-default materials) is legal today
   and stays legal, but is not a scalar palette and cannot sit in
   `scalarLayers`. The scalar-palette test is `arrayBindings: []` and
   `materials: []` together; rule 8.2 and the Palettes prose both spell out
   both halves.
2. **Value-pool liveness.** A pool cell referenced only by a scalar binding
   must keep its pool alive: voxcore's gc, remap, and liveness must treat
   `valueRef` exactly like a materials cell, and `vxl` reporting that counts
   pool references gains a second source.
3. **Rust and TypeScript naming ripple.** Splitting the spec's `Binding` into
   `ArrayBinding` and `ScalarBinding` implies renaming `VoxjPaletteBinding`
   and voxcore's `VoxPaletteBinding` (and its brand type); the final names
   follow open question 2 and get settled in the code phases.
4. **Converter scope is deliberately left at parity.** The format change does
   not require any converter to emit scalar bindings; glTF's
   `KHR_materials_emissive_strength` is the obvious candidate (a shared
   strength becomes a scalar binding instead of a uniform column) but that is
   a capability decision for phase 6, logged in
   [implementation-decisions.md](implementation-decisions.md), not a format
   question.
5. **An object may carry voxels and only scalar layers.** `arrayLayers: []`
   with a non-empty position block is legal (it already is today with
   `layerPaletteRefs: []`); every voxel then takes only scalar contributions
   and defaults. The draft leaves this implicit since no rule changes; worth
   an eye during owner review in case it deserves a sentence.
