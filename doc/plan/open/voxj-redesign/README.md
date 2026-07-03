# Voxj Redesign Migration Plan

Status: **stage 3, finalizing.** All eight decisions are closed; see
[Decisions](#decisions). The executable steps live in
[checklist.md](checklist.md). Everything above the decisions is settled fact from
the format diff and a survey of the current code.

The target format is defined in
[voxel-json-file-format.md](../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md),
already rewritten on this branch in commit `b853623`. Only the spec doc changed;
no Rust code has moved yet.

## What changed in the format

Four semantic deltas, from the diff of the spec against its previous version:

1. **Value pools.** A new shared `main.runtimeState.valuePools` array,
   referenced by index. Each pool is `{ kind, values, min?, max? }`. `kind` is a
   closed vocabulary of eleven tags: `json`, `bool`, `float`, `int`, `string`,
   and six color kinds spanning sRGB and linear space in hex and float forms.
   Hex is sRGB only; a linear color is always float. The bounded kinds, meaning
   `int`, `float`, and the four vector color kinds, require both `min` and `max`,
   each a finite number or the literal string `"none"`. The unbounded kinds carry
   neither.

2. **Palette rewrite.** The old palette was self-contained:
   `{ attributes: string[], data: row[] }`, one inline value per attribute per
   cell. The new palette is `{ bindings: [{ attribute, poolRef }], materials }`.
   Bindings map an attribute name to a value pool. `materials` is column-major:
   one inner array per binding, each of length M, the material count, where
   `materials[b][m]` is a value-index into the pool bound by binding `b`. A
   material is resolved by reading down the columns. No duplicate attribute in a
   palette.

3. **Layers replace the merge stack.** `object.paletteRefs` became
   `object.layerPaletteRefs`, one entry per layer. Two layers may reference the
   same palette. Layers no longer merge; the old "ordered merge, later palette
   overrides earlier" resolution is gone and the meaning of overlapping layers is
   left to the consuming application. A sample is now a material index into that
   layer's palette, not a cell index.

4. **glTF attribute vocabulary.** The recommended attribute names moved to
   glTF's metallic-roughness set. `rgba` became `baseColorFactor`, `metallic`
   became `metallicFactor`, `roughness` became `roughnessFactor`, `occlusion`
   became `occlusionStrength`, `transmission` became `transmissionFactor`, and
   `ior` kept its name. The single `emissive` number split into `emissiveFactor`,
   a color with no alpha, and `emissiveStrength`, a number. The `metallicFactor`
   default flipped from 0 to 1. Colors are now a value pool of one of the color
   kinds, defaulting to `srgba-hex`.

What did **not** change: the position encodings (`raw-json`, `bitmap-base64`,
`hilbert-delta-varint-base64`), the sample codecs (`raw-json`, `rle-json`,
`packed-base64`), the hilbert and varint and bit-packing internals, the
hierarchy and transform model, the edit-state grid, the `ext` namespace, and the
`.voxj` / `.voxjz` container and zip IO. Validation, by contrast, got much
stricter and must be rewritten.

## Crates in the blast radius

The change flows in strict dependency order. Nothing compiles until the whole
chain lands, so the port is one branch even though it is staged into commits.

1. **`voxj`** (`projects/voxel-codecs/voxj`): the serde data model, the wire
   shape. New `VoxjValuePool`, `VoxjValuePoolKind`, `VoxjPaletteBinding` types;
   `VoxjPalette` rewritten to bindings plus column-major materials;
   `VoxjRuntimeState` gains `valuePools`; `VoxjObject.paletteRefs` renamed to
   `layerPaletteRefs`. This crate has no validation today; the redesign may push
   `deny_unknown_fields` and strict typing here.

2. **`voxj-codec`** (`projects/voxel-codecs/voxj-codec`): encode, decode,
   validate, IO. The codecs are untouched, but the width driver
   (`voxj_palette_cell_counts`) must derive the material count M from the new
   palette, `VoxjDecodedObject` renames `paletteRefs` and reframes samples as
   material indices, and the entire validation layer
   (`internal/voxj_validation.rs`, the twelve checks) is rewritten to the twenty
   new rules covering pools, kinds, min/max, bindings, and material ranges.

3. **`voxcore`** (`projects/utilities/voxcore`): the neutral in-memory pivot
   model. Today `VoxPalette` is a rectangular cells-by-attributes grid of untyped
   inline `VoxValue`s, mirroring the old wire format exactly, and `VoxObject`
   forbids referencing the same palette twice. This crate is the crux; see
   [Decisions](#decisions) Q1. It also holds the load-bearing rule
   (no duplicate palette ref) that delta 3 relaxes.

4. **`voxsmith`** (`projects/utilities/voxsmith`): every format converter pivots
   through voxcore. The voxj seam (`internal/voxj`, `convert/voxj`) is rewritten
   to build and read pools plus column-major materials. The glTF pipeline
   (`convert/gltf`, `internal/gltf`, `internal/mesh`) and the shared color
   helpers (`cell_color`, `object_color_ref`, `parse_color_hex`) are pinned to
   the old attribute names and to hex-only color; they move to the glTF vocab and
   pool-kind-aware color. The goxl, mvox, qbcl, and vmax converters each build
   voxcore palettes with old attribute names; vmax is special because it attaches
   two palette refs per object and is the one place the old merge is load-bearing.

5. **`vxl`** (`projects/utilities/vxl`): the CLI. `palette show` and `palette list`
   render the old cell grid and hex-only color; `info` and `hierarchy show`
   print cell counts; `mesh` and the texture presets hardwire the old attribute
   names; the value-pool concept is absent. User-facing "cell" terminology is
   pervasive.

6. **`doc/plan/open/vxl-commands`**: the existing CLI plan documents the old
   palette merge model and defaults `--attribute` to `rgba`. It must be updated
   in lockstep so the plan and the code agree.

Not in scope structurally but worth a glance: `tyt-vmax` and any tyt-side
consumers of voxcore palettes.

## What is settled

Independent of the open decisions, these are true:

- The five block codecs and their internals do not change. Only the documented
  meaning of a sample value changes, from cell index to material index.
- Positions, hierarchy, transforms, edit state, `ext`, and the zip container are
  untouched.
- `version` stays `1`. The redesign did not bump it, so old and new are both
  version 1 and mutually incompatible on the wire. There is no version-gated
  reader. See Q4 for what that implies for existing files.
- `voxj_palette_cell_counts` becomes a material-count helper feeding
  `packed_width`; encode and decode must derive M identically or samples
  mis-slice silently.
- The no-duplicate-palette-ref rule in `voxcore` and the
  `Error::DuplicatePaletteRef` variant encode a constraint the format now
  relaxes, and must go.
- Every test fixture across `voxj-codec`, `voxsmith`, and `vxl` encodes the old
  palette shape and must be rebuilt.

## Decisions

All eight decisions are resolved; each carries a **Decision** line, and the
original framing and recommendation are kept beneath it for the record. The
executable steps are in [checklist.md](checklist.md).

### Q1. voxcore representation: inline, hybrid-typed, or full pool model

**Decision: full pool model with a canonicalized color vocabulary.** `voxcore`
mirrors the wire format, growing pools, bindings, kinds, min/max, and
column-major materials. But it does not carry the wire format's redundant color
encoding variants. The wire has six color kinds spanning hex and float encodings;
`voxcore` reduces these to four canonical color kinds, one per color space and
alpha combination: `srgb`, `srgba`, `linear-rgb`, `linear-rgba`. The wire hex and
float encodings both map onto the matching canonical kind on read, and the writer
picks a wire encoding on write per Q5. So `voxcore`'s
kind vocabulary is nine kinds: `json`, `bool`, `float`, `int`, `string`, and the
four color kinds. This makes round-trips preserve the color value and the space,
while normalizing the on-wire encoding, which is the loss the owner accepted.
The exact in-memory numeric form of a color and how min/max applies to it are a
follow-up under Q5a.

The original options are kept for the record:

This is the load-bearing decision; it sets the blast radius and whether
`voxj -> vox -> voxj` round-trips losslessly. The new format carries information
the current neutral model cannot hold: a value's `kind`, its color space, and
per-attribute `min`/`max`. Three options:

- **A. Inline, unchanged.** `voxcore` keeps its untyped cells-by-attributes grid.
  Pools, kinds, bindings, and column-major materials live only in the `voxj`
  wire layer and the `internal/voxj` seam manufactures them on write and
  discards them on read. Smallest blast radius; converters and glTF and vxl keep
  reading inline `VoxValue`s. Cost: round-trips are lossy (kind, min/max, and the
  srgba-hex-versus-linear-float distinction are invented on write and dropped on
  read), and glTF cannot see color space through the neutral model.

- **B. Full pool model in voxcore.** `voxcore` grows pools, bindings, kinds,
  min/max, and column-major materials, mirroring the wire format. Lossless
  round-trips; kind and color space are available end to end. Cost: the largest
  blast radius; rewrite `VoxPalette` (which uses unsafe struct-of-arrays with
  hand-proved invariants), every converter's build and read path, the glTF
  pipeline, and vxl.

- **C. Hybrid: typed values, no pool indirection (recommended).** `voxcore`
  keeps inline cell values but each value or attribute gains a `kind` tag and
  optional `min`/`max`, so the neutral model can represent an srgba-hex versus a
  linear-rgba-float and carry bounds. Pooling and dedup stay a `voxj`
  serialization concern. Lossless round-trips and color space visible to glTF,
  without inverting the palette to column-major or forcing pool indirection on
  every converter.

Recommendation: **C**, unless we decide lossless round-trip is not required
(then **A**). **B** buys little over **C** at much higher cost.

### Q2. What replaces the merge stack for meshing and color

**Decision: explicit layer selection, defaulting to the first layer; vmax folds
its two palettes into one.** The apps no longer merge across layers. A consumer
that needs one effective material, such as the mesh and material bake, reads a
selected layer, defaulting to the first layer an object references. The mesh and
material commands gain a layer selector. Crucially, `vmax` stops modeling color
and material as two separate layers. Its color palette and material palette
become separate value pools bound as separate attributes within a single
palette, so each voxel samples one material in one layer that already carries
both its color and its PBR attributes. Concretely, the converter builds one
material per distinct color-plus-material combination the voxels actually use,
binds `baseColorFactor` to a color pool and `metallicFactor` and the rest to
their pools, and the object references that one palette on one layer. This
dissolves the two-layer merge problem instead of preserving it, and means
`resolve_used_materials` collapses to a plain material index into the selected
layer's palette. Reconstructing the original vmax color and material indices on
write-back is a converter concern noted in the risks.

**Q2a, confirmed.** The default layer is the first layer an object references,
at index 0.

The original options are kept for the record:

The glTF and mesh atlas define a voxel's material as the tuple of cells merged
across all of an object's palette refs, later ref winning. Delta 3 removes
cross-layer merge. `vmax` is the load-bearing case: it attaches a color palette
and a separate material palette to one object and relies on the merge to combine
them. Two sub-decisions:

- **The rendering convention.** With layers no longer merging in the format, the
  apps still need a voxel's effective material to mesh and color it. Options: (a)
  keep an app-level resolution convention in voxsmith and vxl, merging layers
  left to right exactly as today so behavior is preserved while the format stays
  silent; (b) designate one primary layer for base color and read each attribute
  from whichever layer defines it; (c) require the mesh command to select a
  layer. Recommendation: **(a)**, an explicit app convention, so existing output
  is unchanged and the format-versus-app boundary is clean.

- **vmax's two layers.** Keep vmax emitting two non-merging layers and let the
  app convention combine them, or fold color and material into a single palette
  at conversion. Folding is awkward: the color palette has 255 materials and the
  material palette has N, sampled by independent indices, so a single palette
  would be a cross product. Recommendation: **keep two layers**, combined by the
  app convention from the previous bullet.

### Q3. Attribute vocabulary ownership

**Decision: neutral strings plus a shared glTF-name constant table.** `voxcore`
attribute names stay free strings. One shared constant module holds the glTF
names and every converter, the glTF pipeline, and vxl import from it, so custom
attributes such as vmax `shadows` and mvox `flux` stay expressible. The full
rename map, the `emissive` split into `emissiveFactor` plus `emissiveStrength`,
and the `metallicFactor` default flip from 0 to 1 are adopted everywhere.

The original framing is kept for the record:

Every converter writes attribute names into voxcore palettes and glTF reads them
back, so all producers and consumers must agree on one vocabulary. Options:
`voxcore` stays neutral free strings with a single shared glTF-name constant
module that every converter, the glTF pipeline, and vxl import; or `voxcore`
enforces a closed glTF-attribute enum. Recommendation: **neutral strings plus a
shared constant table**, set to the glTF names, so custom attributes (vmax
`shadows`, mvox `flux`) stay expressible. Independently, confirm the full rename
map, the `emissive` split into `emissiveFactor` plus `emissiveStrength`, and the
`metallicFactor` default flip from 0 to 1 are adopted everywhere.

### Q4. Backward compatibility and existing files

**Decision: clean break.** Old `.voxj` files stop parsing under the new model.
No reader for the old shape, no upgrade tool, and every checked-in fixture is
regenerated to the new shape.

The original framing is kept for the record:

`version` stays 1, so old `.voxj` files simply will not parse under the new
model. Confirm this is a clean break: no old files in the wild to preserve, no
migration/upgrade path required, and all checked-in fixtures are regenerated.
Recommendation: **clean break**, regenerate fixtures, no reader for old files.
If real old files exist we need a separate one-shot upgrade tool, which changes
scope.

### Q5. Default color kind and min/max policy on write

**Decision: default the on-wire color encoding to float, behind a `--color-format`
option limited to `hex` and `float` for now; choose min/max from a per-attribute
default table.** The writer emits float color kinds by default, so base color
round-trips without 8-bit quantization. A `--color-format` option lets the user
ask for `hex` instead where the color is in a space that has a hex kind. Only
sRGB has hex kinds in the format, so a linear color always serializes as float
regardless of the option. This does change today's hex-first behavior and will
churn the golden color assertions, which is accepted. The integer color kinds
were dropped from the format entirely (see the format-spec change), so hex and
float are the whole color-encoding vocabulary; explicit linear-space authoring
stays deferred.

For bounds, the writer uses a per-attribute default table:
`metallicFactor`, `roughnessFactor`, `occlusionStrength`, `transmissionFactor`
at `0..1`; `ior` at `1..none`; `emissiveStrength` at `0..none`; color float
kinds at `0..1` for sRGB and `0..none` for linear to allow HDR; anything
unrecognized at `none..none`. Values are still validated within the chosen
bounds.

**Q5a, confirmed.** `voxcore` stores each color as float components in natural
range: sRGB and srgba at `0..1` with 8-bit hex mapped by dividing by 255, and
linear allowing values above 1 for HDR.

The original framing is kept for the record:

The writer must invent information the neutral model may lack. Two conventions to
fix:

- **Default color kind.** Emit `srgba-hex` to match today's 8-bit sRGB behavior
  and keep golden hex assertions stable, or emit a float or linear kind for
  fidelity. Recommendation: **`srgba-hex` default**, with fidelity kinds only
  where a source genuinely carries them, decided under Q1.

- **min/max defaults.** Bounded kinds require bounds the sources rarely carry.
  Recommendation: a **per-attribute default table**: `metallicFactor`,
  `roughnessFactor`, `occlusionStrength`, `transmissionFactor` at `0..1`; `ior`
  at `1..none`; `emissiveStrength` at `0..none`; color float kinds at `0..1`;
  anything unknown at `none..none`. Values still validated within the chosen
  bounds.

### Q6. Strictness placement and pool construction

**Decision (owner did not object to the recommendation).** serde gets
`deny_unknown_fields` on the closed structures so unknown keys reject at parse;
`voxj-codec` implements the twenty validation rules as named checks for
`vxl validate` reporting; the writer builds one pool per `(kind, bounds)` deduped
document-wide and dedups identical values within a pool and identical materials
within a palette. Settled at the code level during execution.

### Q7. CLI surface: terminology and pool visibility

**Decision: rename cell to material, keep values resolved inline.**
`--show-cells`, the `cells` columns, `cellCount`, and the cell wording in
`palette show`, `info`, and `hierarchy show` become material terminology, a
deliberate breaking output change. `palette show` and `palette list` keep
resolving and printing each material's values inline; a dedicated value-pool
inspector is deferred.

The original framing is kept for the record:

Two user-facing choices. Rename `--show-cells`, the `cells` columns, and
`cellCount` to material terminology now, a breaking output change, or keep
"cell" as the user word. And whether `palette show` and `palette list` should
surface value pools (pool index, kind, bounds) or keep resolving values inline
per material as today, possibly with a new pool inspector. Recommendation:
**rename to "material"** to match the format, and **keep values resolved inline**
in the existing commands, deferring a dedicated pool inspector.

### Q8. Scope of this effort: faithful port or feature expansion

**Decision: faithful port with parity, plus a log of deferred capabilities.**
The effort gets every crate compiling and passing on the new format with
behavior unchanged. New capabilities the format now enables but that we are not
building yet are recorded in
[Deferred capabilities](#deferred-capabilities-log) as we notice them, so the
follow-up work is captured without bloating this change.

## Deferred capabilities log

New capabilities the redesigned format enables, deferred out of the faithful
port and captured here for follow-up:

- Expose both color spaces in the CLI. The faithful port ships `--color-format`
  limited to `hex` and `float`; letting the user author or select linear-space
  color is deferred. (The integer color kinds are not deferred; they were removed
  from the format.)
- Read `ior` and `transmissionFactor` on glTF import; today the importer drops
  them.
- Preserve glTF `KHR_materials_emissive_strength` through import rather than
  collapsing emissive to a single scalar.
- A value-pool inspector surface in the CLI, for example `vxl palette pools`,
  showing pool kinds, bounds, and sharing.
- Author or convert into non-default color encodings on request, for example
  emitting `linear-rgba-float` for HDR fidelity rather than the `srgba-hex`
  default.

## Execution shape

The [checklist](checklist.md) follows the dependency order below, each step
compiling as far in isolation as possible with its fixtures rebuilt alongside it:

1. `voxj` data model: new pool and binding types, palette rewrite, runtime-state
   and object field changes, serde strictness.
2. `voxj-codec`: material-count helper, decoded-object rename, validation
   rewrite, fixtures.
3. `voxcore`: representation per Q1, relax the duplicate-ref rule, kind/min-max
   plumbing, gc and validate updates, fixtures.
4. `voxsmith` voxj seam: pool build and read, column-major materials, layer refs
   and material-index samples.
5. `voxsmith` color helpers and glTF pipeline: glTF vocab, pool-kind-aware color,
   the emissive split, the metallic default.
6. `voxsmith` goxl, mvox, qbcl, vmax converters: shared vocab table, vmax folded
   into one palette.
7. `vxl`: cell-to-material terminology, color rendering by pool kind, attribute
   vocab, encoding-option docs.
8. `doc/plan/open/vxl-commands`: update the reference to the new palette model.

## Test and fixture strategy

Every fixture across the three downstream crates encodes the old palette shape
and several tests assert now-removed rules such as duplicate-palette-ref
rejection and `rgba` hex format. Plan to rebuild fixtures alongside each crate's
change rather than in a final sweep, and to add coverage for the genuinely new
surface: pool kind validation, min/max presence and bounds, column-major
material ranges, two layers on one palette, and the stricter null and
unknown-key rejection. The byte-exact round-trip tests in the converters
(`goxl`, `mvox`, `qbcl`, `vmax`) constrain pool ordering, dedup, and color-kind
choice to be deterministic and information-preserving; Q1 and Q5 decide whether
they can pass unchanged or must be regenerated.
