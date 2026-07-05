# Voxj Follow-Up Capabilities Plan

Status: **open.** This plan captures the follow-up work the voxj redesign enabled
or exposed but the faithful port left out. It began from the redesign plan's
[deferred capabilities log](../../closed/voxj-redesign/README.md#deferred-capabilities-log)
and now covers three tracks: exposing color spaces in the CLI, closing the glTF
import fidelity gap, and verifying vmax material round-trips end to end. The
value-pool inspector from the deferred log is deliberately dropped; see
[Not in scope](#not-in-scope). All three decisions are resolved. The executable
steps live in [checklist.md](checklist.md), with a per-session resume prompt in
[continue-voxj-followups.md](continue-voxj-followups.md).

The predecessor port is closed: every crate builds, lints, and tests green on the
redesigned format. This plan is additive and each track is independent, so this is
three loosely-coupled tracks rather than one branch.

The authoritative format is
[voxel-json-file-format.md](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md).
It does not change here; every track is consumer-side.

## The tracks

**Track A: color spaces and encodings in the CLI** (deferred items 1 and 5). The
port ships `--color-format` limited to `hex` and `float`, both sRGB. voxcore
already carries four canonical color kinds, `srgb`, `srgba`, `linear-rgb`, and
`linear-rgba`, and the wire carries their float encodings, but no CLI path lets a
user select a linear-space color, and no converter emits one. The follow-up
exposes linear space and the non-default encodings, for example emitting
`linear-rgba-float` for HDR fidelity. Per Q1 this extends the existing
`--color-format` flag rather than adding a second flag.

**Track B: glTF import fidelity** (deferred items 2 and 3). glTF is a mesh source,
so it enters only through `vxl voxelize` and leaves only through `vxl mesh`. The
importer `mesh_material_from_gltf` reads base color, metallic, roughness, and the
emissive color into a six-field `MeshMaterial`, and `build_palette` binds exactly
those six attributes. It drops `KHR_materials_ior` and `KHR_materials_transmission`
outright, since no field holds them, and it hardwires `emissive_strength` to `1`
because it never reads `KHR_materials_emissive_strength`, so a glTF authored at a
non-unit strength imports at unit strength. Emissive is already two separate
bindings, `emissiveFactor` and `emissiveStrength`, so the real gap is the unread
strength extension, not a collapse to one scalar. The bake and export side is
attribute-agnostic and already knows the `ior` and `transmissionFactor` defaults
through `default_scalar`, so the fix is mostly import-side; per Q2 the export path
also writes the three KHR extensions back for a symmetric round-trip.

**Track C: vmax material round-trip verification** (new, a testing track). Confirm
that materials survive a full trip through voxj, both directions. A vmax source
folds color and every material attribute into one palette and stores the exact
`VoxelMaxMaterial` list in its `voxel-max` ext, so `vmax -> voxj -> vmax` should
restore materials from the ext byte-exactly. A glTF source has no ext, so
`glb -> voxj -> vmax` exercises the writer's synthesized derive path, which joins
materials by signature under the 256-material budget. This track adds end-to-end
coverage over both pipelines rather than new behavior, and files any fidelity gap
it finds as its own checklist item. It depends on Track B only for `ior` and
`transmission` on a glTF source; every other attribute round-trips today.

## Not in scope

The value-pool inspector (`vxl palette pools`) from the deferred log is dropped.
`palette show` and `palette list` keep resolving values inline per material; a
dedicated pool view is not planned here. The other four deferred items are covered
by Tracks A and B.

## Crates in the blast radius

The tracks do not chain; each names its own crates.

**Track A.**
- `voxsmith`: `ColorFormat` and `VoxjFileBuilder::color_format` pick the wire
  encoding, today only across sRGB hex and float. A linear selection must reach
  the color pool the converter or voxelizer builds, since a linear pool always
  serializes as float. voxsmith's existing `ColorSpace` type is the palette
  quantization space (Oklab, Lab, Rgb), unrelated to sRGB-versus-linear, which is
  one reason Q1 extends `--color-format` rather than adding a `--color-space` flag.
- `vxl`: `VoxjColorFormat` (`hex`/`float`) on `VoxjEncodingOptions`, extended with
  linear values.

**Track B.**
- `voxsmith`: `MeshMaterial` (`internal/mesh/mesh_material.rs`) gains `ior` and
  `transmission` fields; `mesh_material_from_gltf` (`convert/gltf/from_gltf_bytes.rs`)
  reads the three KHR extensions; `build_palette`
  (`convert/voxelize/voxelize_mesh.rs`) binds two more pools; and, per Q2
  symmetric, `build_material` (`internal/gltf/material_document.rs`) emits the
  three KHR extensions on export.
- `gltf_attributes` already defines `IOR`, `TRANSMISSION_FACTOR`, and
  `EMISSIVE_STRENGTH`, and `default_scalar` already carries their bake defaults, so
  no new names are needed.
- `vxl`: only `voxelize` is affected; `mesh` already tolerates the new bindings.

**Track C.**
- No production code unless a gap is found. Exercises `vxl voxelize`,
  `vxl to voxj`, and `vxl to vmax`, and the voxsmith vmax converter
  (`convert/vmax`, `internal/vmax`). New tests may live in `voxsmith` at the
  converter level or `vxl` at the command level.

## What is settled

- The wire format, its color kinds, and validation are fixed. Every track is
  consumer-side; none touches the spec or bumps `version`.
- voxcore already stores the four canonical color kinds losslessly, so Track A
  surfaces what the model holds rather than extending it.
- `ior`, `transmissionFactor`, and `emissiveStrength` already exist as
  `gltf_attributes` constants honored on bake and export, so Track B is an
  import-side asymmetry, not a new attribute vocabulary.
- vmax already round-trips rich materials through its ext at the unit-test level
  (`to_vmax_file` `round_trips_rich_materials`) and synthesizes an ext-free source
  (`synthesizes_a_file_without_an_ext`), so Track C is end-to-end confirmation, not
  a rewrite.

## Decisions

### Q1. CLI surface for color space and encoding (Track A)

**Decision: extend `--color-format`.** The color-space choice folds into the
existing encoding flag as new values rather than a second flag. voxsmith already
uses `ColorSpace` for the quantization space, so a `--color-space` flag would read
as that unrelated concept; folding into `--color-format` avoids the collision and
keeps one flag for how a color serializes.

The original framing is kept for the record:

- **A. Extend `--color-format`** with a `linear-float` value, folding space and
  encoding into one flag.
- **B. Add an orthogonal `--color-space`** flag (`srgb`/`linear`) composing with
  `--color-format`, rejecting `hex` plus `linear`.
- **C. Authoring-only** via `--define-attribute` on `voxelize`, no writer flag.

The owner chose **A**; the `ColorSpace` naming collision above is the deciding
factor.

### Q2. glTF import fidelity: how far to carry it (Track B)

**Decision: symmetric.** The import half is the floor: add `ior` and
`transmission` to `MeshMaterial` and read the three KHR extensions on import. On
top of that the exporter also writes `KHR_materials_ior`,
`KHR_materials_transmission`, and `KHR_materials_emissive_strength` back from
`build_material`, so a `voxelize -> mesh` trip reproduces the source glTF's flat
factors and no source, glb included, loses material information the format can
hold.

Note the export wrinkle this accepts: today the glTF exporter bakes every
attribute, even metallic and roughness, into texture atlases and writes no flat
scalar factors on the material. Symmetric export therefore adds a new behavior,
emitting flat glTF extension values, that no other attribute uses yet. It stays
scoped to these three KHR extension fields.

The original options are kept for the record:

- **A. Import-only.** Land the values in voxj, where the bake can already pack
  them into a texture channel, and leave the glTF material export as today.
- **B. Symmetric.** Also write the three KHR extensions back from
  `build_material`.

The owner chose **B**, since Track C's aim is faithful round-trips and A would
leave glb the one source whose factors cannot make it back out.

### Q3. Scope and sequencing

**Decision: independent tracks, any order.** Each track lands on its own branch
with its own fixtures. The only coupling is that Track C's `glb -> voxj -> vmax`
fidelity for `ior` and `transmission` depends on Track B; run Track C's
vmax-source pipeline first, or accept those two attributes as a known gap until B
lands.

## Execution shape

1. Track A: extend `--color-format` with linear values per Q1, thread the choice
   to the pool the writer and voxelizer build, add HDR and linear round-trip
   coverage.
2. Track B: add the `MeshMaterial` fields, read the three KHR extensions on
   import, bind them in `voxelize_mesh`, and emit them from `build_material`; add
   a full-material glTF round-trip.
3. Track C: run `vmax -> voxj -> vmax` and `glb -> voxj -> vmax` over real assets,
   assert materials restore, and file any gap as a new item.

## Test and fixture strategy

Track A needs a linear and an HDR color round-trip and a rejection test for an
illegal encoding-and-space combination. Track B needs a glTF fixture authoring
`KHR_materials_emissive_strength`, a transmission factor, and an `ior`, asserting
each survives import and a subsequent export. Track C needs a
representative `.vmax` with rich materials, meaning metallic, roughness, emission,
and the dispersion-derived `ior` and `transmission`, plus a textured `.glb`, and
asserts the voxj intermediate carries the folded material attributes and the
rebuilt vmax restores them. Each track rebuilds only its own fixtures; the wire
format does not change.
