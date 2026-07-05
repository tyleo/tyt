# Implementation decisions

_Part of the [Voxj Follow-Up Capabilities Plan](../README.md)._

Code-level decisions made while executing the [checklist](../checklist.md),
recorded as they land. The plan-level decisions and their rationale live in the
[README](../README.md#decisions); this log is for the finer implementation choices
a reviewer of the Rust would want explained, for example how the linear value
threads onto `--color-format`, how `ior` and `transmission` default when a glTF
omits them, and what the vmax verification found at each hop.

No work has landed yet. Add a section under the relevant track as its first chunk
lands.

## Track A: color spaces and encodings in the CLI

Landed the linear color space as a third `--color-format` value.

- **Space and encoding fold into one enum value.** `VoxjColorFormat` and the
  voxsmith `ColorFormat` gain a single `linear-float` / `LinearFloat` variant
  beside `hex` and `float`, rather than a space flag composed with an encoding
  flag. This makes the one illegal pairing, linear plus hex, unrepresentable, so
  checklist item 3's "reject an illegal combination" needs no runtime guard: the
  flag surface never lets it be expressed. The existing
  `color_format_rejects_an_unknown_encoding` test keeps a bare `linear` invalid.
- **The sRGB-to-linear transfer lives in voxsmith, not ty-math.** The decode is a
  private helper (`srgb_to_linear`, with `decode_rgb` / `decode_rgba`) in
  `voxj_value_pool_from_vox_value_pool.rs`, matching the standard sRGB EOTF that
  ty-math's own `srgb_to_linear` uses. It is not added to ty-math because Track
  A's declared blast radius is voxsmith and vxl, and ty-math is published to
  crates.io, where a new public method would diverge from the `0.1.x` on the
  registry. The decode works on the `f64` components directly, never routing
  through the 8-bit path, so no float precision is lost.
- **One write-time hook covers both commands.** The conversion happens where the
  writer emits each `VoxjValuePool`, which `vxl to voxj` and `vxl voxelize` both
  reach through `write_voxj_document`, so threading the choice once serves both.
  A pool voxcore already stores in a linear kind is emitted as float under every
  choice, unchanged; only the two sRGB kinds vary. Alpha carries no gamma, so the
  decode passes the straight alpha through untouched.
- **Two surfaces, one story (item 4).** `--define-attribute` declares a pool's
  color kind when authoring, including `linear-rgb` / `linear-rgba` for HDR
  authoring; `--color-format` picks how the written pools serialize and may
  decode the sRGB kinds to linear at write. The `VoxjColorFormat` doc comment
  states this split so the authoring and writing surfaces do not read as rivals.
- **Coverage.** voxsmith tests pin the transfer against reference linear values,
  prove an sRGB pool decodes to `linear-rgb-float`, keep the straight alpha,
  round-trip sRGB through linear and back within epsilon, preserve an HDR
  component above 1 on an already-linear pool, and leave the sRGB `float` default
  byte-for-byte unchanged. vxl parses `--color-format linear-float`.

## Track B: glTF import fidelity

Landed the import half: glTF now reads `ior`, `transmissionFactor`, and
`KHR_materials_emissive_strength` into the voxelized palette. The symmetric export
half stays for a follow-up chunk; see the last point.

- **The import half is its own chunk.** Q2's symmetric decision has two
  independent halves: import reads the three KHR extensions, export writes them
  back. The import half is the plan's stated floor and closes exactly the two
  import entries in the redesign deferred log, so it lands first as a
  self-contained, green change. The export half from `build_material` is a new
  behavior with real design weight, emitting flat glTF extension values that no
  other attribute uses and reconciling with how the emissive bake already folds
  strength into the emissive texture, so it gets its own chunk and decision entry
  rather than being rushed in alongside import.
- **The three KHR accessors are feature-gated.** The `gltf` crate hides
  `Material::ior`, `Material::transmission`, and `Material::emissive_strength`
  behind `KHR_materials_ior`, `KHR_materials_transmission`, and
  `KHR_materials_emissive_strength`, so voxsmith's `gltf` dependency now enables
  those three. They gate only serde fields on `gltf-json`, so no new crate enters
  the lock.
- **Absent extensions import as spec neutrals.** Each accessor returns an
  `Option`, so `mesh_material_from_gltf` falls back to the spec default when the
  extension is missing: ior `1.5`, transmission `0`, and emissive strength `1`.
  The strength fallback preserves the prior hardwired `1`, so a plain
  `emissiveFactor` still imports at unit strength; only an authored strength now
  survives. `MeshMaterial::flat` seeds the same ior `1.5` and transmission `0`, so
  a flat or fill material is a neutral dielectric.
- **`build_palette` binds eight attributes.** Two float pools join the six:
  `ior` bounded `1..none` through `float_above(1.0)` and clamped with `.max(1.0)`,
  and `transmissionFactor` bounded `0..1` through `bounded_float(0.0, 1.0)` and
  clamped with `.clamp(0.0, 1.0)`, matching the redesign plan's per-attribute
  bounds. `MaterialKey` gains the two scalar bit patterns so materials differing
  only in ior or transmission stay distinct, the same raw-bits identity the other
  scalars use. Constant defaults do not split materials, so an ordinary glTF
  voxelizes to the same material count as before, now over an eight-binding
  palette.
- **Deferred-log entries were import-only.** The two redesign entries described
  the import drop and the pinned strength, both now fixed, so they collapse to a
  single pointer at the plan rather than staying open. The pointer notes the
  import half has landed, leaving the symmetric export as the remaining Track B
  work.
- **Coverage.** A new glb fixture authors the three extensions and asserts each
  survives import at its authored value, and a plain glb asserts the neutral ior
  and transmission defaults. The existing binding-list assertion grows from six
  attributes to eight. The full import-and-export round-trip fixture (item 6) and
  the export emission (item 4) are the next chunk.

## Track C: vmax material round-trip verification

_Pending._
