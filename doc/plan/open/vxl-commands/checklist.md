# Implementation Checklist

Tracks building the commands in this plan. Start from the [README](README.md)
for the overview, the per-command pages under [reference/](reference/), and the
[design notes](reference/design-notes.md) for rationale. Code-level decisions
made while building are logged in
[implementation decisions](reference/implementation-decisions.md). Check items
off as they land.

## Ground rules

- `vxl` stays independent of `tyt-common` and `tyt-injection`. Use `std::fs` and
  the codec crates already wired in `Cargo.toml` (`voxcore`, `voxj-codec`,
  `voxsmith`, `vmax-codec`). The tyt FBX and material commands are behavioral
  models only, not dependencies.
- Follow the existing command house style: one `clap` `Parser` struct per file
  in `src/commands/`, re-exported from `commands/mod.rs`, dispatched from
  `vxl.rs`; one `Dependencies` trait method per operation in `dependencies.rs`
  with the concrete impl behind the `impl` feature under `implementation/`. Use
  `to voxj` (`commands/voxj.rs`, `implementation/to_voxj.rs`) as the template.
- Each `#[arg]` starts with `value_name`. Enumerated options are `ValueEnum`
  types, one per file under `utilities/`, re-exported. Outputs are
  `Option<PathBuf>` defaulted from the input stem. Booleans use the settable
  `--ext` style.

## Shared infrastructure

- [x] `MeshFormat` `ValueEnum` (to support `fbx` | `obj` | `gltf`) with
      `from_path` extension inference, mirroring `Format::from_path`. Only
      implement `fbx` at first.
- [x] `--select-index` object selector parser: integer or `a-b` range,
      repeatable, union over all values. See
      [conventions](reference/conventions.md).
- [x] `--select` hierarchy-path glob object selector: a node path selects its
      subtree, repeatable, union over all values. See
      [conventions](reference/conventions.md).
- [x] `--atlas` layout `ValueEnum`: palette (one texel per palette entry,
      shareable) and unwrap (per-mesh UV) layouts. See [mesh](reference/mesh.md).
- [x] `--texture-map` channel parser: channel sources (`R`/`G`/`B`/`A` = `attr`
      | `1-attr` | `attr.r`/`.g`/`.b`/`.a` color component | `0` | `1` |
      `computed-occlusion`) and the RGBA packing, with `smoothness` accepted as
      `1-roughness`. See [mesh](reference/mesh.md).
- [x] `--texture` preset packings (albedo, orm, metallic-roughness,
      metallic-smoothness, mse, emissive, occlusion, computed-occlusion,
      roughness, smoothness). See [mesh](reference/mesh.md).
- [x] `--define-attribute` binding types: `ColorComponent`, `AttributeType`
      (`scalar` | `color`), and `AttributeBinding` (`name palette key [type]`).
      The clap multi-value wiring lands with `mesh`. See
      [mesh](reference/mesh.md).
- [x] Shared voxj encoding options (`--format`, `--encoding-preset`,
      `--position-encoding`, `--sample-encoding`) in `VoxjEncodingOptions`,
      flattened by `to voxj` and `voxelize`; `--ext`/`--edit-state` stay on
      `to voxj`.
- [x] `ValueEnum`s for the palette ops: quantize method, color space, dither,
      and `palette show` format (`auto` | `swatch` | `swatch-value` | `value`).

## Commands

### mesh ([reference/mesh.md](reference/mesh.md))

- [ ] `Mesh` command struct, dispatch, and single-object pure-geometry output;
      error when the selection is not exactly one object.
- [ ] `--to` / `--from`, `--scale` (meters per voxel, default `1.0`;
      centimeter-native `fbx` writes `100 * scale`), `--method`,
      `--vertex-computed-occlusion` with `--computed-occlusion-strength`,
      `--computed-occlusion-min-brightness`, and
      `--computed-occlusion-color-space`, `--atlas`,
      `--select` / `--select-index`.
- [ ] Material maps: `--texture <name> [path]` presets (albedo, orm,
      metallic-roughness, metallic-smoothness, mse, emissive, occlusion,
      computed-occlusion, roughness, smoothness), `--texture-map <path>
      <channels>`, and `--define-attribute <name> <palette-index> <key> [type]`,
      default paths from the mesh stem.
- [ ] `Dependencies::mesh` and its impl.

### material ([reference/material.md](reference/material.md))

- [ ] `Material` command sharing the mesh map flags, bake-only with no geometry.
- [ ] `--atlas` shared with `mesh`; atlas derivation identical per mode; verify
      byte-for-byte parity.
- [ ] Require at least one map; otherwise list the maps and exit non-zero.

### voxelize ([reference/voxelize.md](reference/voxelize.md))

- [ ] `Voxelize` command; `--from`, mutually exclusive
      `--side-length` | `--voxel-size`.
- [ ] Reuse the voxj writer options; optionally record `--voxel-size` as the
      node scale.

### palette ([reference/palette/](reference/palette/))

- [ ] `palette list` (+ `--layout`): index, attributes, cell count, referencing
      objects.
- [x] `palette show` (+ `--json`): `--index` / `--attribute` / `--format`. See
      the V2 follow-ups in [palette show](reference/palette/show.md) for the
      broader version.
- [ ] `palette quantize`: `--count` with median-cut / octree / kmeans, space,
      dither with `--select` / `--select-index`, and the multi-attribute merge
      rule. Accept a full document or a bare palette JSON; dither only with a
      document.
- [ ] `palette remap`: `--target` (JSON `palettes` array) or `--target-index`,
      `--target-attribute`, space, dither with `--select` / `--select-index`,
      and the same merge rule. Accept a full document or a bare palette JSON
      input, the `--target` shape; dither only with a document.

### hierarchy show ([reference/hierarchy/show.md](reference/hierarchy/show.md))

- [ ] Tree render with DAG / instancing and unplaced-node markers.
- [ ] `pattern` glob; `--show-transforms` / `--show-bounds` / `--show-extents`;
      collapse flags; `--layout`.

### validate ([reference/validate.md](reference/validate.md))

- [x] Implement the spec validation checklist; non-zero exit on failure;
      `--layout` report.

### info ([reference/info.md](reference/info.md))

- [ ] Report version, per-object bounds / voxel count / encodings, palette
      attribute sets and cell counts, `editState` and `ext` presence, and root /
      instanced / unplaced nodes; `--layout`.

## Finishing

- [ ] `--layout` output on `list`, `hierarchy show`, `validate`, and `info`;
      `palette show` keeps its own `--json`.
- [ ] Help text and `clap_complete` completions cover the new commands.
- [ ] Tests per command, following the existing test style.

## Deferred (see [Future](reference/design-notes.md))

- [ ] Scene-assembly mode: hierarchy-node selection with baked transforms and
      instancing.
- [ ] Single-object vs whole-document output nuance, and multi-object mesh
      layout in one file.
- [ ] stdin / stdout via `-`; dry-run for destructive palette ops.
