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
  models only, not dependencies. Mesh reading for `mesh` and `voxelize` goes
  through voxsmith, which gains a `gltf` feature gating the `gltf` crate, rather
  than a vxl-level mesh dependency.
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

- [x] `MeshFormat` `ValueEnum` of `gltf` | `glb` with `from_path` extension
      inference, mirroring `Format::from_path`. glTF is the only mesh format for
      now, read via the `gltf` crate. (A prior scaffold held an `fbx` variant;
      retarget it to glTF.)
- [x] `--select-index` object selector parser: integer or `a-b` range,
      repeatable, union over all values. See
      [conventions](reference/conventions.md).
- [x] `--select` hierarchy-path glob object selector: a node path selects its
      subtree, repeatable, union over all values. See
      [conventions](reference/conventions.md).
- [x] `--atlas` layout `ValueEnum`: palette (one texel per material of the baked
      layer's palette, placed at its material index, shared by every mesh on that
      palette) and unwrap (per-mesh UV) layouts. See [mesh](reference/mesh.md).
- [x] `--texture-map` channel parser: channel sources (`R`/`G`/`B`/`A` = `attr`
      | `1-attr` | `attr.r`/`.g`/`.b`/`.a` color component | `0` | `1` |
      `computed-occlusion`) and the RGBA packing, with `smoothness` accepted as
      `1-roughness`. See [mesh](reference/mesh.md).
- [x] `--texture` preset packings (albedo, orm, metallic-roughness,
      metallic-smoothness, mse, emissive, occlusion, computed-occlusion,
      roughness, smoothness). See [mesh](reference/mesh.md).
- [x] `--define-attribute` binding: `ColorComponent` and `AttributeBinding`
      (`name=key`, a pure rename alias). The type is not declared: `mesh` reads it
      from the key's value pool in the meshed layer's palette at bake, a color
      pool exposing components and a scalar pool read whole, and an absent
      attribute follows the format's unbound-default rule (a glTF built-in bakes
      its spec default, a custom key errors). The former `AttributeType` enum and
      the `[:type]` suffix are gone; the file is the single source of type truth.
      See [mesh](reference/mesh.md).
- [ ] `--vertex` / `--vertex-map` carrier: preset-to-attribute-name mapping
      (`albedo`/`computed-occlusion` → `COLOR_0`, the scalar/packed presets →
      `_NAME`, `palette-index` → `_PALETTEINDEX` over a per-mesh used-combos
      table, `palette-layers` → `_PALETTEINDEX0..n` over per-layer palettes),
      reusing the `--texture-map` channel parser and `--define-attribute`; emit
      glTF vertex attributes (`COLOR_0` standard, `_NAME` custom) and write the
      `PaletteData` JSON per `--palette-storage`. See [mesh](reference/mesh.md).
- [x] `ResourceStorage` `ValueEnum` (`embedded` | `external` | `both`) backing
      `--texture-storage` and `--palette-storage`, defaulting per target
      (`embedded` for `.glb`, `external` for `.gltf`): embed images in the glb
      chunk / gltf data URI and the palette JSON under `extras.vxl`, write
      external `.png` and `-palette.json` files, or both. See
      [mesh](reference/mesh.md). (Image storage shipped with `mesh`; the palette
      JSON reuse lands with the vertex carriers.)
- [x] Shared voxj encoding options (`--format`, `--encoding-preset`,
      `--position-encoding`, `--sample-encoding`, `--color-format`) in
      `VoxjEncodingOptions`, flattened by `to voxj` and `voxelize`;
      `--ext`/`--edit-state` stay on `to voxj`.
- [x] `ValueEnum`s for the palette ops: quantize method, color space, dither,
      and `palette show` format (`auto` | `swatch` | `swatch-value` | `value`).
- [ ] Shared palette-reduction engine and a flattened options group
      (`--method` / `--space` / `--dither`), reused by `palette quantize`,
      `palette remap` (space/dither), and `voxelize`'s `--max-palette-materials`,
      on the one material-follows-color rule (a count bounds materials; a merged
      material takes its cluster representative's whole material). Landed: the
      `PaletteReductionOptions` group and voxsmith's `reduce_palette` with all
      three methods (`median-cut` / `octree` / `kmeans`) in oklab/lab/rgb via
      `remove_material`+`gc`, plus `--dither` (`floyd-steinberg` / `ordered`) as a
      per-voxel remap in 3D raster order. Pending: `quantize` / `remap` will
      reuse the engine.

## Commands

### mesh ([reference/mesh.md](reference/mesh.md))

- [x] `Mesh` command struct, dispatch, and single-object pure-geometry output;
      error when the selection is not exactly one object. The mesher lives in
      voxsmith (`object_to_mesh_geometry` plus `object_to_glb_bytes` /
      `object_to_gltf_bytes`) behind the `gltf` feature; vxl stays a thin CLI.
- [x] `--to` / `--from` (`gltf` | `glb`), `--voxel-size` (meters per voxel, default
      `1.0`, baked into every vertex; glTF is meter-native), `--method`, and the
      `--select` / `--select-index` object selectors, `--select` matched through
      the shared `pathspec` gitignore engine like `hierarchy show`.
- [ ] `--atlas` and the `--computed-occlusion-strength` /
      `--computed-occlusion-min-brightness` / `--computed-occlusion-color-space`
      tuning (land with the texture / occlusion maps below).
- [x] Material maps: `--texture <preset>` presets (albedo, orm,
      metallic-roughness, metallic-smoothness, mse, emissive, occlusion,
      computed-occlusion, roughness, smoothness) and the `pbr` bundle, repeatable;
      `--texture-name <preset> <file-name>` / `--texture-name-prefix <prefix>`
      naming; `--texture-map <file-name> <channels>`; `--define-attribute
      <name>=<key>`; `--layer` (0-based
      object-layer index whose materials this mesh bakes, default `0`); and
      `--texture-storage` (embedded / external / both); default names from the
      output stem, unique per bake. (The `--atlas palette` path;
      `computed-occlusion` errors until the unwrap atlas lands.)
- [ ] Vertex attribute maps: `--vertex <preset>` presets (albedo →
      `COLOR_0`, computed-occlusion → `COLOR_0` darken, metallic / roughness /
      emissive / occlusion / smoothness → scalar `_NAME`, orm / mse /
      metallic-roughness / metallic-smoothness → packed `_NAME`, palette-index →
      `_PALETTEINDEX` over a per-mesh used-combos table, palette-layers →
      `_PALETTEINDEX0..n` over per-layer palettes), the `PaletteData` JSON
      written per `--palette-storage` (extras / `-palette.json` / both),
      `--vertex-target <preset> <target>` overriding a preset's attribute, and
      `--vertex-map <target> <channels>` reusing the `--texture-map` channel
      grammar and `--define-attribute`.
- [x] `Dependencies::resolve_objects` and `mesh_object` and their impls, the
      flag-agnostic split that replaces the planned single `Dependencies::mesh`:
      the impl resolves the selectors to object indices and meshes by index,
      while the command owns the exactly-one policy and its flag-named errors.

### material ([reference/material.md](reference/material.md))

- [ ] `Material` command sharing the mesh map flags, bake-only with no geometry.
- [ ] `--atlas` shared with `mesh`; atlas derivation identical per mode; verify
      byte-for-byte parity.
- [ ] Require at least one map; otherwise list the maps and exit non-zero.

### voxelize ([reference/voxelize.md](reference/voxelize.md))

- [x] `Voxelize` command; `--from` (`gltf` | `glb`), mutually exclusive
      `--resolution` | `--voxel-size` (clap `ArgGroup`,
      `required = true`).
- [x] `--fill-mode` `solid` (default) | `surface`; `--fill-color` (default
      `white`, a `#RRGGBBAA` hex or name), used by `solid` only and rejected with
      `surface`. Surface is a hollow shell; per-voxel color sampling from the
      glTF material is deferred, so surface uses the flat color for now. (Shipped
      MVP; superseded by the material-mode work below.)
- [x] Reuse the voxj writer options; record `--voxel-size` (meters per
      voxel) as the node scale, leaving `--resolution` at scale `1`.
- [x] voxsmith `gltf` feature gating the `gltf` crate and a mesh-to-`VoxMain`
      voxelizer; `Dependencies::voxelize` and its impl call it, then write
      through `VoxjFileBuilder` like `to voxj`. glTF Y-up is converted to the
      Z-up document convention; the inverse `mesh` command must mirror it.

Material sampling (see [voxelize](reference/voxelize.md) and
[design notes](reference/design-notes.md)):

- [x] `--material-mode auto | per-primitive | per-texel | flat` (default
      `auto`), replacing the shipped flat-only coloring and its
      `--fill-color`-with-`--fill-mode surface` guard. `per-texel` and
      texture-aware `auto` still fall back to `per-primitive` until the texel
      sampler lands below.
- [x] `per-primitive`: one material per source glTF material from the PBR factors
      (`baseColorFactor`, `metallicFactor`, `roughnessFactor`, `emissiveFactor`,
      `emissiveStrength`, `occlusionStrength`), matching what `mesh` bakes.
- [x] `--fill-color #RRGGBBAA` (omitted for the default): the whole object under
      `flat`, the `solid`-fill interior under the sampling modes; a set color is
      rejected on a sampling-mode surface. An omitted interior adopts its nearest
      surface material.
- [x] `--max-palette-materials <n> | none` (default `256`) via the shared
      reduction engine; expose `--method` / `--space` / `--dither` on `voxelize`.
      Landed for
      all three methods (`median-cut` / `octree` / `kmeans`) with a stderr note,
      and both `--dither` modes (`floyd-steinberg` / `ordered`).
- [x] `per-texel`: UV interpolation, image decode, area-average over the voxel
      footprint, epsilon-merge of near-identical tuples, and a `solid`-interior
      fallback to the nearest surface material. `auto` becomes texture-aware here.
      Base color, metallic, roughness, emissive, and occlusion all sample their
      own glTF textures on one scatter pass, each map reading the TEXCOORD set it
      declares (sRGB for base color and emissive, linear data for
      metallic-roughness and occlusion), area-averaged and merged; a map a
      material lacks keeps its flat factor.

### palette ([reference/palette/](reference/palette/))

- [x] `palette list` (+ `--layout`): index, attributes, material count,
      referencing objects.
- [x] `palette show` (+ `--json`): `--index` / `--attribute` / `--format`. See
      the V2 follow-ups in [palette show](reference/palette/show.md) for the
      broader version.
- [ ] `palette quantize`: `--count` with median-cut / octree / kmeans, space,
      dither with `--select` / `--select-index`, and the material-follows-color
      reduction rule (`--count` bounds materials; a merged material takes its
      cluster representative's whole material). Accept a full document or a bare
      palette JSON; dither only with a document.
- [ ] `palette remap`: `--target` (JSON `palettes` array) or `--target-index`,
      `--target-attribute`, space, dither with `--select` / `--select-index`,
      and the same material-follows-color rule (a remapped voxel adopts the
      target material's whole set of attribute values). Accept a full document or
      a bare palette JSON input, the `--target` shape; dither only with a
      document.

### hierarchy show ([reference/hierarchy/show.md](reference/hierarchy/show.md))

- [x] Tree render with DAG / instancing and unplaced-node markers, plus
      `--collapse-instances`. Markdown tree only; no `--layout`.
- [x] `pattern` glob with `--collapse-ancestors` / `--collapse-descendants`.
- [x] `--show-transforms`, local and world.
- [x] `--show-{edit,runtime}-{origins,bounds,extents}`, the edit and runtime
      grids relative to the placing node, origins with a local/world space, an
      absent edit grid printing `null`, an empty object's runtime grid a
      zero-size box at its origin.
- [x] `--show-layers`, one child per layer labeled by its palette index with its
      material count.

### validate ([reference/validate.md](reference/validate.md))

- [x] Implement the spec validation checklist; non-zero exit on failure;
      `--layout` report.

### info ([reference/info.md](reference/info.md))

- [ ] Report version, per-object bounds / voxel count / encodings, palette
      attribute sets and material counts, `editState` and `ext` presence, and root /
      instanced / unplaced nodes; `--layout`. Landed: the command, dispatch,
      `Dependencies::info`, and a markdown / pretty-json / compact-json `--layout`
      report covering version, per-object bounds, voxel count, palette attribute
      sets and material counts, and `editState` / `ext` presence, with tests. Pending:
      per-object encodings and the root / instanced / unplaced node breakdown.

## Finishing

- [x] `--layout` output on `list`, `validate`, and `info`; `palette show` keeps
      its own `--json`, and `hierarchy show` prints only its tree.
- [x] Help text and `clap_complete` completions cover the new commands.
- [ ] Tests per command, following the existing test style.

## Deferred (see [Future](reference/design-notes.md))

- [ ] Scene-assembly mode: hierarchy-node selection with baked transforms and
      instancing.
- [ ] Single-object vs whole-document output nuance, and multi-object mesh
      layout in one file.
- [ ] stdin / stdout via `-`; dry-run for destructive palette ops.
- [x] Move the palette reduction from vxl into voxsmith as a general operation
      (public `reduce_palette` + plain enums; vxl maps its clap `ValueEnum`s like
      `FillMode` / `MaterialMode` do). See
      [implementation decisions](reference/implementation-decisions.md).
- [x] Adopt richer typed color types (one struct per space with `from_` / `to_`
      conversions) in `ty-math`, in place of the ad-hoc `[u8;4]` -> `[f64;3]`
      math, so spaces cannot be mixed. Landed with the generic `Mesh` (glTF is
      one reader into it). See
      [implementation decisions](reference/implementation-decisions.md).
