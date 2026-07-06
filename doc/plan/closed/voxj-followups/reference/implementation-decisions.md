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

Landed both halves. Import: glTF reads `ior`, `transmissionFactor`, and
`KHR_materials_emissive_strength` into the voxelized palette. Export: the mesh
writes the three back as flat KHR extensions on the material.

- **The two halves landed as two chunks.** Q2's symmetric decision has two
  independent halves: import reads the three KHR extensions, export writes them
  back. Import landed first as the plan's stated floor, closing the two import
  entries in the redesign deferred log. Export followed as its own chunk, since
  it emits flat glTF extension values no other attribute uses and it reworks the
  emissive bake.
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
  single pointer at the plan rather than staying open.
- **Emissive strength: unfold, not fold.** The owner chose to emit strength as a
  flat factor over the old fold-into-texture. `emissive_color_bytes` now bakes
  the raw `emissiveFactor` color and the material carries
  `KHR_materials_emissive_strength`, so glTF applies the strength. An HDR
  strength above `1` now survives instead of clamping into the `[0, 1]` texel;
  the tradeoff is that per-material strength collapses to one flat value.
- **Flat factors from the first used material.** glTF carries `ior`,
  `transmissionFactor`, and `emissiveStrength` per material, not per texel, so
  `material_extensions` reads the first used material as the representative and
  writes one flat value for the whole mesh. `material_scalar` reuses the bake's
  attribute reader.
- **Emitted only when non-default.** Each extension is written only when its
  value differs from the spec default (ior `1.5`, transmission `0`, strength
  `1`), so a plain export carries no redundant extensions, and `extensionsUsed`
  lists only what is written.
- **Coverage.** Import: a glb fixture authors the three extensions and asserts
  each survives import, and a plain glb asserts the neutral ior and transmission
  defaults. Export: the same fixture voxelizes, meshes back out with no maps, and
  re-imports, confirming each factor rides on the exported material. The
  emissive-bake test flips to assert the raw factor bakes without strength, and
  the binding-list assertion grows from six attributes to eight.

## Track C: vmax material round-trip verification

Verified both pipelines against real assets in `scratch/`: `energy-reactor.vmax`,
whose second palette carries `ior`, `transmissionFactor`, and `absorption`
alongside metallic, roughness, emissive, and shadows, and `energy-turret.glb`, a
textured mesh. Neither asset is committed, so the durable coverage is the unit
and integration tests below rather than a fixture check.

- **vmax -> voxj -> vmax round-trips materials byte-exactly.** `to voxj` folds
  color and every material attribute into the palette and also stores the exact
  `VoxelMaxMaterial` list in the `voxel-max` ext; `to vmax` restores from that
  ext rather than re-deriving, so the rebuilt palette values match the source
  and the ext's `palettes` block is byte-identical across a second conversion.
  The folded palette holds one entry per used (color, material) pair, so its
  count exceeds the ext's raw material count; an unused Voxel Max slot survives
  in the ext but never enters the fold.
- **glb -> voxj -> vmax hit a color-budget gap, now fixed.** A voxelized glb
  folds thousands of sampled colors into `baseColorFactor`. `voxelize`'s material
  reduction (`reduce_palette`) clustered materials but never compacted the value
  pools, and `VoxMain::gc` compacts only id pools, never the values inside a
  pool, so the color pool kept every sampled color and overflowed Voxel Max's
  255-color budget. `to vmax` errored on the first voxel referencing a color cell
  past 255, at every material cap.
- **The fix is a pool-prune primitive the reduction opts into.** `voxcore` gains
  `VoxMain::prune_value_pools`, which drops every pool entry no material
  references, renumbers the survivors densely, and rewrites the value-indices
  that point at them. References union across every palette, so a shared entry
  survives while any one material uses it. A pool no material references is left
  whole, since `validate` requires every pool non-empty. `reduce_palette` gains a
  `keep_unused_values` argument and prunes after reducing unless it is set;
  `to voxj` never reduces, so faithful loads keep every value they were given.
- **The CLI knob is `--keep-unused-values`, off by default.** `voxelize` gains
  `--keep-unused-values`, default false, so the reduction drops unused values
  unless asked to keep them. The one `keep_unused_values` name threads through
  the flag, `PaletteReduction`, and `reduce_palette`, so no polarity flips.
  Dropping is the expected default; passing `--keep-unused-values` keeps every
  sampled value, reproducing the old overflow. The prune runs inside the
  reduction, so it pairs with `--max-palette-materials`: capping to 255 and
  dropping yields at most 255 colors, which writes vmax cleanly.
- **Coverage.** `voxcore` tests prune-and-remap, cross-palette union retention,
  and the fully-referenced no-op; `voxsmith` tests that a fused-away color and its
  tag leave the pools under `drop_unused` and stay under the opt-out; `vxl` tests
  the reduction drops by default and that `--keep-unused-values` opts out. End to
  end, `voxelize --max-palette-materials 255 energy-turret.glb` then `to vmax`
  writes a 255-color vmax, while `--keep-unused-values` still overflows.
- **Command-level pipeline tests skipped at close.** End-to-end tests that drive
  `voxelize`, `to voxj`, and `to vmax` as a pipeline (checklist item 5) were
  deliberately not added: they would need committed vmax and glb fixtures, and
  the round-trips are already proven by the unit and integration tests plus the
  manual runs recorded here. The owner closed the plan on this basis.
