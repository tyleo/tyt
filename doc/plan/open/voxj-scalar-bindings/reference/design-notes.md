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
3. **Layer-attached constant refs**, `{ property, poolRef, valueRef }` on
   the layer. Works, but adds a second supply path for property values
   outside palettes; rejected to keep palettes the only property carrier.
4. **Knob-as-material with a `composes` palette flag**, a format-defined
   layer merge. Carries the knob in channel content: a phantom `M = 1`
   material whose validator-forced uniform channel must track the voxel
   count.
5. **`M = 0` palettes with an empty-channel carve-out in rule 11.**
   Superseded by the two-list split, which removes the vestigial channel
   structurally instead of by rule exception.
6. **Hierarchy-node attachment** for per-instance variation. Makes material
   resolution path-dependent through the instancing DAG and demands
   inheritance rules; out of scope. Nothing in this design forecloses adding
   it later.
7. **Top-level named constants registry.** Scalar-binding palettes plus their
   pools are already enumerable; a display name is an editor concern the wire
   format does not need.
8. **Scalar palettes as a palette class** (`arrayBindings: []` plus
   `materials: []` as the only legal `scalarLayers` target, with dead-data
   and duplicate-entry rejection). Dropped in owner review 2026-07-15: there
   are just palettes; a scalar layer uses exactly its palette's scalar
   bindings, and either layer list may reference any palette, repeats
   included.
9. **App-defined cross-layer combination.** The first draft kept today's
   posture; owner review 2026-07-15 replaced it with a canonical override
   order, each property taken from the last layer that supplies it. (The
   fixed scalars-then-arrays order it first came with fell with the two-list
   model, alternative 10.)
10. **Two layer lists, one arity per list** (`arrayLayers` / `scalarLayers`,
    the first settled object shape). A mixed palette had to be listed in
    both lists, and the fixed scalars-then-arrays precedence could not
    express a whole-object override of a per-voxel value. Replaced in owner
    review 2026-07-15 by the single ordered `layers` list with channels
    derived from palette shape (README decision 10).
11. **An explicit per-layer `sampled` flag** on the single list. Keeps the
    channel count local to the object but adds a second source of truth that
    can disagree with the palette's shape; the derived sampled-iff-`M > 0`
    rule has nothing to disagree with.
12. **A channel for every layer**, degenerate for scalar-only palettes.
    Vestigial data, and it resurrects the `M = 0` carve-out in rule 11 that
    decision 3 exists to avoid.

## Strain noticed while drafting

1. **A fully shadowed layer is legal.** Under the override order a repeated
   `layers` palette leaves the earlier entry's contributions, channel
   included, supplying nothing; the format does not police pointlessness.
2. **Value-pool liveness.** A pool cell referenced only by a scalar binding
   must keep its pool alive: voxcore's gc, remap, and liveness must treat
   `valueRef` exactly like a materials cell, and `vxl` reporting that counts
   pool references gains a second source.
3. **Rust and TypeScript naming ripple.** Splitting the spec's `Binding` into
   `ArrayBinding` and `ScalarBinding` implies renaming `VoxjPaletteBinding`
   and voxcore's `VoxPaletteBinding` (and its brand type); the field names
   are settled (`array*` / `scalar*`), and the final Rust type names get
   settled in the code phases.
4. **Converter scope is deliberately left at parity.** The format change does
   not require any converter to emit scalar bindings; glTF's
   `KHR_materials_emissive_strength` is the obvious candidate (a shared
   strength becomes a scalar binding instead of a uniform column) but that is
   a capability decision for phase 6, logged in
   [implementation-decisions.md](implementation-decisions.md), not a format
   question.
5. **An object may carry voxels and only unsampled layers.** `layers`
   referencing only `M = 0` palettes with a non-empty position block is
   legal (an empty `layers` already is today with `layerPaletteRefs: []`);
   `voxelSamples` then has no channels, and every voxel takes only scalar
   contributions and defaults. The draft leaves this implicit since the
   channel rule covers it; worth an eye during owner review in case it
   deserves a sentence.
6. **Channel count is derived.** A reader counts an object's channels by
   dereferencing each layer's palette, and giving a scalar-only palette its
   first material changes the channel count of every referencing object.
   Accepted in owner review 2026-07-15: rule 11 already dereferences each
   channel's palette for `M`, and `M` edits already invalidate referencing
   sample blocks (packed widths and index ranges move with `M`).
7. **The property rename reaches CLI and identifier surface.** Renaming
   `attribute` to `property` (README decision 11) implies renaming the
   by-attribute maps, vxl's `attribute_binding` module, `attribute_ref` /
   `attribute_selector`, `attribute_names`, and the `--define-attribute`
   flag; final identifier and flag names settle in the code phases.
