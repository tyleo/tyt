# Voxj Follow-Up Capabilities Checklist

Tracks the three independent follow-up tracks from the [README](README.md): color
spaces in the CLI, glTF import fidelity, and a value-pool inspector. Unlike the
redesign checklist, these phases do not chain; pick any phase whose decision is
resolved. Every item is unchecked because no work has started.

Log non-obvious code-level choices in
[reference/implementation-decisions.md](reference/implementation-decisions.md) as
they land, the same way the redesign and vxl-commands plans do.

## Ground rules

- Each phase is a standalone branch off `main`. Do not start a phase until its
  README decision (Q1, Q2, or Q3) is resolved with the owner; the steps below
  assume the recommended resolution and must be revised if the owner chooses
  otherwise.
- Run `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D
  warnings` before every commit; the pre-commit hook enforces both.
- Follow the repo style: Rust edition 2024, consolidated nested `use`, one public
  type per file named in snake_case, doc comments on public items, comments
  wrapped to 80 columns and ASCII-only.
- Rebuild only the touched crate's fixtures; the wire format does not change, so
  there is no cross-crate fixture churn and the workspace stays green throughout.

## Phase A: color spaces and encodings in the CLI

Assumes Q1 resolves to an orthogonal `--color-space` flag.

- [ ] Add a `--color-space` option (`srgb`/`linear`, default `srgb`) beside
      `--color-format` on the voxj-writing path, following the `VoxjColorFormat`
      pattern on `VoxjEncodingOptions`.
- [ ] Reject `--color-format hex` with `--color-space linear` at parse or
      validation, since the format has no linear hex kind.
- [ ] Thread the chosen space to the color pool the writer and `voxelize` build,
      so a linear selection emits a `linear-rgb` or `linear-rgba` pool that always
      serializes as float.
- [ ] Reconcile `--define-attribute`'s existing linear kinds with the new flag on
      one story, and document which surface authors versus converts color space.
- [ ] Add coverage: an sRGB-to-linear conversion round-trip, an HDR component
      above 1 preserved through `linear-rgba-float`, and the illegal
      hex-plus-linear rejection.

Gate: `vxl` writes a linear-space document, an HDR color survives the round-trip,
and the illegal combination is rejected with a clear message.

## Phase B: glTF import fidelity

Assumes Q2 resolves to first-class fields bound unconditionally.

- [ ] Add `ior` and `transmission` fields to `MeshMaterial`, defaulting to the
      per-attribute table (`ior` `1`, `transmission` `0`) so absent glTF factors
      import as the neutral value.
- [ ] Read `KHR_materials_emissive_strength` in `mesh_material_from_gltf` into
      `emissive_strength` instead of the hardwired `1`, and read the glTF `ior`
      (`KHR_materials_ior`) and `transmissionFactor` (`KHR_materials_transmission`)
      into the new fields.
- [ ] Bind `ior` and `transmissionFactor` in `voxelize_mesh` and `build_palette`
      with the per-attribute bounds (`transmissionFactor` `0..1`, `ior` `1..none`).
- [ ] Keep the export and material-document paths symmetric so a voxj material
      carrying these attributes writes them back into the glTF.
- [ ] Update the deferred-capabilities note in `mesh_material_from_gltf`'s doc
      comment, and remove the matching entries from the redesign plan's deferred
      log with a pointer here.
- [ ] Add a glTF import fixture authoring `KHR_materials_emissive_strength`, a
      transmission factor, and an `ior`, asserting each survives import and a
      subsequent export.

Gate: a glTF with emissive strength, transmission, and refraction round-trips
through voxelize and export without losing those factors.

## Phase C: value-pool inspector

Assumes Q3 resolves to a dedicated `palette pools` subcommand.

- [ ] Add pool-enumeration and pool-sharing accessors to voxcore, and to
      `voxj-codec` if the read model needs them, so a caller can list pools and
      the bindings that reference each.
- [ ] Add a `vxl palette pools` subcommand beside `palette show` and
      `palette list`, addressing a document the same way, with text and JSON
      output.
- [ ] Render one row per pool: index, kind, bounds, value count, and the
      `palette:attribute` bindings drawing from it; render a shared pool once with
      all its bindings.
- [ ] Add fixtures for the text and JSON layouts, including a pool shared by two
      bindings and a bounded `int` or `float` pool showing its min and max.

Gate: `vxl palette pools` lists every pool with its kind, bounds, and sharing on
a new-shape document, in text and JSON.
