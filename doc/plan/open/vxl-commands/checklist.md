# Implementation Checklist

Tracks building the commands in this plan. Start from the [README](README.md)
for the overview, the per-command pages under [reference/](reference/), and the
[design notes](reference/design-notes.md) for rationale. Check items off as they
land.

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

- [ ] `MeshFormat` `ValueEnum` (to support `fbx` | `obj` | `gltf`) with
      `from_path` extension inference, mirroring `Format::from_path`. Only
      implement `fbx` at first.
- [ ] Object selector parsers: `--select-index` (integer or `a-b` range) and
      `--select` name glob, both repeatable, union over all values. See
      [conventions](reference/conventions.md).
- [ ] Material-map model: `--atlas` palette (one texel per palette entry,
      shareable) and unwrap (per-mesh UV) layouts; preset packings and the
      `--texture-map` channel parser (`R`/`G`/`B`/`A` = `attr` | `1-attr` | `0`
      | `1`), with `smoothness` accepted as `1-roughness`. See
      [mesh](reference/mesh.md).
- [ ] Shared voxj writer options (`--format`, `--optimize`,
      `--position-encoding`, `--sample-encoding`, `--ext`, `--edit-state`)
      factored for reuse by `voxelize`.
- [ ] `ValueEnum`s for the palette ops: quantize method, color space, dither,
      and `palette show` format (`auto` | `swatch` | `string`).

## Commands

### mesh ([reference/mesh.md](reference/mesh.md))

- [ ] `Mesh` command struct, dispatch, and per-object pure-geometry output.
- [ ] `--to` / `--from`, `--scale` (meters per voxel, default `1.0`;
      centimeter-native `fbx` writes `100 * scale`), `--method`,
      `--vertex-computed-occlusion` with `--computed-occlusion-strength`,
      `--computed-occlusion-min-brightness`, and
      `--computed-occlusion-color-space`, `--atlas`,
      `--select` / `--select-index`.
- [ ] Material maps: `--texture <name> [path]` presets (albedo, orm,
      metallic-roughness, metallic-smoothness, mse, emissive, occlusion,
      computed-occlusion, roughness, smoothness) and `--texture-map <path>
      <channels>`, default paths from the mesh stem.
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

- [ ] `palette list` (+ `--json`): index, attributes, cell count, referencing
      objects.
- [ ] `palette show` (+ `--json`): `--index` / `--attribute` / `--format`.
- [ ] `palette quantize`: `--count` with median-cut / octree / kmeans, space,
      dither, and the multi-attribute merge rule.
- [ ] `palette remap`: `--target` (JSON `palettes` array) or `--target-index`,
      `--target-attribute`, space, dither, and the same merge rule.

### hierarchy show ([reference/hierarchy/show.md](reference/hierarchy/show.md))

- [ ] Tree render with DAG / instancing and unplaced-node markers.
- [ ] `pattern` glob; `--show-transforms` / `--show-bounds` / `--show-extents`;
      collapse flags; `--json`.

### validate ([reference/validate.md](reference/validate.md))

- [ ] Implement the spec validation checklist; non-zero exit on failure;
      `--json` report.

### info ([reference/info.md](reference/info.md))

- [ ] Report version, per-object bounds / voxel count / encodings, palette
      attribute sets and cell counts, `editState` and `ext` presence, and root /
      instanced / unplaced nodes; `--json`.

## Finishing

- [ ] `--json` output on `list`, `show`, `hierarchy show`, `validate`, `info`.
- [ ] Help text and `clap_complete` completions cover the new commands.
- [ ] Tests per command, following the existing test style.

## Deferred (see [Future](reference/design-notes.md))

- [ ] Scene-assembly mode: hierarchy-node selection with baked transforms and
      instancing.
- [ ] Single-object vs whole-document output nuance, and multi-object mesh
      layout in one file.
- [ ] stdin / stdout via `-`; dry-run for destructive palette ops.
