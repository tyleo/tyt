# Voxj Follow-Up Capabilities Plan

Status: **open.** This plan captures the five capabilities the redesigned voxj
format now enables but the faithful port deliberately left out, recorded in the
voxj-redesign plan's
[deferred capabilities log](../../closed/voxj-redesign/README.md#deferred-capabilities-log).
None of the decisions here are resolved; the [Decisions](#decisions) below are
open questions with recommendations, not settled calls. The executable steps
live in [checklist.md](checklist.md), and a per-session resume prompt in
[continue-voxj-followups.md](continue-voxj-followups.md).

The predecessor port is closed: every crate builds, lints, and tests green on
the redesigned format. This plan is additive. Nothing here is load-bearing for
the format's correctness; each capability is an independent enhancement that can
land on its own, so this is not one branch but three loosely-coupled tracks.

The authoritative format is
[voxel-json-file-format.md](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md).
Use it for any color-space, encoding, or validation detail this plan leaves
implicit. The format itself does not change; every track is consumer-side.

## The capabilities

Five deferred items, grouped into three tracks by the surface they touch:

**Track A: color spaces and encodings in the CLI** (deferred items 1 and 5).
The port ships `--color-format` limited to `hex` and `float`, both sRGB. voxcore
already carries four canonical color kinds, `srgb`, `srgba`, `linear-rgb`, and
`linear-rgba`, and the wire carries their float encodings, but no CLI path lets a
user author or select a linear-space color, and no converter emits one. The
follow-up exposes linear-space color and the non-default encodings, for example
emitting `linear-rgba-float` for HDR fidelity rather than the sRGB default.

**Track B: glTF import fidelity** (deferred items 2 and 3). The importer
`mesh_material_from_gltf` reads base color, metallic, roughness, and the emissive
color, but drops `ior` and `transmissionFactor` and hardwires `emissive_strength`
to `1`. A glTF authored with `KHR_materials_emissive_strength`, a transmission
factor, or a refraction index loses that information on import, even though the
export and bake paths already understand those attributes. The follow-up reads
them through, so a glTF round-trips its full metallic-roughness material.

**Track C: a value-pool inspector in the CLI** (deferred item 4). `palette show`
and `palette list` resolve each material's values inline and never surface the
shared value pools directly, so pool kinds, bounds, and cross-palette sharing are
invisible. The follow-up adds a read-only inspector, for example
`vxl palette pools`, that lists each pool's index, kind, bounds, and the bindings
that draw from it.

## Crates in the blast radius

Unlike the redesign, these tracks do not chain; each names its own crates.

**Track A.**
- `voxsmith`: `ColorFormat` and `VoxjFileBuilder::color_format` pick the wire
  encoding today but only across sRGB hex and float. A linear selection has to
  reach the color pool the converter or voxelizer builds, since a linear pool
  always serializes as float regardless of the encoding flag. `--define-attribute`
  already accepts the four color kinds on `voxelize`, so a linear source is
  partly reachable there; the gap is a coherent user story across the color
  writers.
- `vxl`: `VoxjColorFormat` (`hex`/`float`) on `VoxjEncodingOptions`, and the
  `AttributeType` kind vocabulary `--define-attribute` already exposes.

**Track B.**
- `voxsmith`: `MeshMaterial` gains `ior` and `transmission` fields;
  `mesh_material_from_gltf` populates them and reads
  `KHR_materials_emissive_strength` into `emissive_strength`; `voxelize_mesh`
  binds the two new attributes into the palette; the glTF export path
  (`object_to_gltf_document` and the material document) keeps the round-trip
  symmetric.
- `gltf_attributes` already defines `IOR`, `TRANSMISSION_FACTOR`, and
  `EMISSIVE_STRENGTH`, so no new attribute names are needed.

**Track C.**
- `vxl`: a new `pools` command under `palette`, following the existing
  `palette show` and `palette list` structure and their text-and-JSON rendering.
- `voxcore`, and possibly `voxj-codec`: pool-enumeration and pool-sharing
  accessors, if the read model does not already expose which bindings reference
  each pool.

Out of scope: the format spec and the wire model. These are all consumers of the
existing format.

## What is settled

Independent of the open decisions:

- The wire format, its color kinds, and validation are fixed. Every capability
  here is a consumer-side enhancement; none touches the spec or bumps `version`.
- voxcore already stores the four canonical color kinds losslessly, so Track A
  surfaces what the model can already hold rather than extending it.
- `ior`, `transmissionFactor`, and `emissiveStrength` already exist as
  `gltf_attributes` constants and are honored on export and bake, so Track B is
  an import-side asymmetry, not a new attribute vocabulary.
- The three tracks are independent and individually shippable; there is no forced
  order and no phase blocks another.

## Decisions

Open. Each carries framing and a recommendation; resolve with the owner before
executing that track, then record the call as a **Decision** line the way the
closed redesign plan does.

### Q1. CLI surface for color space and encoding (Track A)

`--color-format` currently picks the sRGB wire encoding, `hex` or `float`. Three
ways to expose linear space:

- **A. Extend `--color-format`** with a `linear-float` value, folding space and
  encoding into one flag.
- **B. Add an orthogonal `--color-space`** flag (`srgb`/`linear`) that composes
  with `--color-format`, rejecting `hex` plus `linear` since the format has no
  linear hex kind.
- **C. Authoring-only**, treating color space purely as a `--define-attribute`
  concern on `voxelize` and adding no writer flag, leaving conversion of an
  existing document's color space out of scope.

Recommendation: **B**, an orthogonal `--color-space`, because it mirrors the
format's own factoring of space and encoding and keeps the illegal
hex-plus-linear combination a single clear error.

### Q2. glTF import fidelity: how far to carry it (Track B)

The import gap is `ior`, `transmissionFactor`, and
`KHR_materials_emissive_strength`. Two questions: whether to add `ior` and
`transmission` as first-class `MeshMaterial` fields with per-attribute default
bindings, matching how metallic and roughness are carried, or to bind them only
when the source glTF declares them; and whether the export side then always emits
them so a voxj-authored material shows up in the glTF, or only when present.
Recommendation: **first-class fields with the per-attribute default table**,
`transmissionFactor` at `0..1` and `ior` at `1..none`, bound unconditionally,
matching the existing metallic-roughness treatment and keeping import and export
symmetric.

### Q3. Value-pool inspector shape (Track C)

A dedicated `vxl palette pools` subcommand, or a `--show-pools` flag on the
existing `palette show`. And what a pool row carries: index, kind, bounds, value
count, and the `palette:attribute` bindings that draw from it. Recommendation:
**a dedicated `palette pools` subcommand** with text and JSON output like its
siblings, so the existing per-material views stay unchanged and the pool view can
show document-wide sharing a per-palette command cannot.

### Q4. Scope and sequencing

The three tracks are independent. Confirm they land as separate branches in any
order, each with its own fixtures, rather than one combined change.
Recommendation: **three independent branches**, since nothing couples them and
each has a distinct blast radius and test surface.

## Execution shape

Each track is a short independent sequence; the [checklist](checklist.md) keeps
them in separate phases so a session can pick up any one without touching the
others.

1. Track A: reconcile the color-space CLI surface per Q1, thread it to the pool
   the writer and voxelizer build, and add HDR and linear round-trip coverage.
2. Track B: add the `MeshMaterial` fields, read the three glTF attributes on
   import, bind them in `voxelize_mesh`, keep export symmetric, and add a
   full-material glTF round-trip.
3. Track C: add the pool accessors if missing and the `palette pools` command,
   with text and JSON fixtures.

## Test and fixture strategy

Track A needs a linear and an HDR color round-trip and a rejection test for the
illegal hex-plus-linear combination. Track B needs a glTF fixture that authors
`KHR_materials_emissive_strength`, a transmission factor, and an `ior`, asserting
each survives import and a subsequent export. Track C needs pool-listing goldens
in both text and JSON, including a pool shared by two bindings and a bounded
`int` or `float` pool showing its min and max. Each track rebuilds only its own
fixtures; there is no cross-crate fixture churn because the wire format does not
change.
