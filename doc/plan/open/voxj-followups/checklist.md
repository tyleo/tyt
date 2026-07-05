# Voxj Follow-Up Capabilities Checklist

Tracks the three follow-up tracks from the [README](README.md): color spaces in
the CLI, glTF import fidelity, and vmax material round-trip verification. The
phases do not chain; pick any one. Every item is unchecked because no work has
started.

Log non-obvious code-level choices in
[reference/implementation-decisions.md](reference/implementation-decisions.md) as
they land, the same way the redesign and vxl-commands plans do.

## Ground rules

- Each phase is a standalone branch off `main`. All three decisions are settled:
  Track A extends `--color-format` (Q1), Track B is symmetric (Q2), and the tracks
  are independent (Q3).
- Run `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D
  warnings` before every commit; the pre-commit hook enforces both.
- Follow the repo style: Rust edition 2024, consolidated nested `use`, one public
  type per file named in snake_case, doc comments on public items, comments
  wrapped to 80 columns and ASCII-only.
- Rebuild only the touched crate's fixtures; the wire format does not change, so
  there is no cross-crate fixture churn and the workspace stays green throughout.

## Phase A: color spaces and encodings in the CLI

Per Q1, the color space folds into `--color-format` as new values, not a new flag.

- [x] Extend `VoxjColorFormat` on `VoxjEncodingOptions` with a linear value (for
      example `linear-float`) beside `hex` and `float`, so one flag names both the
      space and the encoding.
- [x] Thread the chosen space to the color pool the writer and `voxelize` build,
      so a linear selection emits a `linear-rgb` or `linear-rgba` pool that always
      serializes as float, and an sRGB selection is unchanged.
- [x] Map hex to sRGB only; a linear value implies float, since the format has no
      linear hex kind. Reject an illegal combination clearly if the flag surface
      lets one be expressed.
- [x] Reconcile with `--define-attribute`'s existing color kinds so authoring and
      writing agree on one story, and document which surface does which.
- [x] Add coverage: an sRGB-to-linear conversion round-trip, an HDR component
      above 1 preserved through the linear float encoding, and the sRGB default
      left unchanged.

Gate: `vxl` writes a linear-space document and an HDR color survives the
round-trip, with the sRGB default and existing goldens unchanged.

## Phase B: glTF import fidelity

Both halves are settled: symmetric per Q2, so import reads the extensions and
export writes them back.

- [ ] Add `ior` and `transmission` fields to `MeshMaterial`, defaulting to the
      per-attribute table (`ior` `1.5`, `transmission` `0`) so absent glTF factors
      import as the neutral value.
- [ ] In `mesh_material_from_gltf`, read `KHR_materials_emissive_strength` into
      `emissive_strength` instead of the hardwired `1`, and read
      `KHR_materials_ior` and `KHR_materials_transmission` into the new fields.
- [ ] Bind `ior` and `transmissionFactor` in `build_palette`
      (`voxelize_mesh.rs`) with the per-attribute bounds (`transmissionFactor`
      `0..1`, `ior` `1..none`), extending the six-attribute palette to eight.
- [ ] Emit `KHR_materials_ior`, `KHR_materials_transmission`, and
      `KHR_materials_emissive_strength` from `build_material`
      (`material_document.rs`) so a `voxelize -> mesh` trip reproduces the source
      glTF's flat factors. This adds flat-factor export, which no attribute uses
      today (all others bake to textures); keep it scoped to these three fields.
- [ ] Update the deferred-capabilities note in `mesh_material_from_gltf`'s doc
      comment, and remove the two matching entries from the redesign plan's
      deferred log with a pointer here.
- [ ] Add a glTF import fixture authoring `KHR_materials_emissive_strength`, a
      transmission factor, and an `ior`, asserting each survives import and a
      subsequent export.

Gate: a glTF with emissive strength, transmission, and refraction imports without
losing those factors and round-trips back out.

## Phase C: vmax material round-trip verification

A verification track: run the pipelines over real assets, prove materials survive,
and file any gap as a new item. No production change unless a gap is found.

- [ ] Identify a representative `.vmax` with rich materials (metallic, roughness,
      emission/`sic`, the dispersion-derived `ior` and `transmission`, and the
      vmax-only `shadows` and `absorption`) and a textured `.glb`; note where the
      assets live.
- [ ] Run `vmax -> voxj -> vmax`: `vxl to voxj model.vmax out.voxj`, then
      `vxl to vmax out.voxj rebuilt.vmax`. Confirm the voxj palette folds color
      and the material attributes into one palette, and the rebuilt vmax restores
      the exact `VoxelMaxMaterial` list from the `voxel-max` ext.
- [ ] Run `glb -> voxj -> vmax`: `vxl voxelize model.glb out.voxj`, then
      `vxl to vmax out.voxj from_glb.vmax`. Confirm the synthesized derive path
      maps baseColor, metallic, roughness, and emissive into vmax materials within
      the 256-material budget.
- [ ] Compare materials at each hop and record which attributes survive, which are
      vmax-only (`shadows`, `absorption`, dispersion), and which depend on Track B
      (`ior` and `transmission` from a glb source).
- [ ] Turn both pipelines into end-to-end tests, at the converter or command
      level, so the round-trips stay covered.
- [ ] File any fidelity gap found as a new checklist item under the track it
      belongs to.

Gate: both pipelines run clean and materials are shown to survive each hop, or the
gaps are filed with a reproduction.
