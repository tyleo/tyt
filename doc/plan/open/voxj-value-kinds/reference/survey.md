# voxj value kinds survey

Line-level inventory behind the [checklist](../checklist.md), taken at the
keyboard on 2026-08-01. Line numbers drift as work lands; re-grep before
editing. The [README](../README.md) holds the design; this file holds what
the code says today, including five places where it says something the
README's blast radius does not (see
[corrections](#corrections-to-the-readmes-blast-radius)).

## Constraints the iterations run under

- Crate graph: `voxj` feeds `voxj-codec`, which feeds `voxsmith`, `vxl`,
  and `tyt-vmax`. `voxcore` depends on neither voxj crate and feeds
  `voxsmith` and `vxl`. `voxsmith` feeds `vxl` and `tyt-vmax`; `vxl` feeds
  `tyt`. Five crates change source; seven must compile again by the end.
  `tyt-vmax` and `tyt` are recompile-only: neither names a value pool type
  in code.
- The pre-commit hook runs `cargo fmt --all -- --check` and
  `cargo clippy --workspace --all-targets -- -D warnings`, so a
  half-migrated tree cannot commit through it; its header documents
  `git commit --no-verify` as the bypass. At survey time this clone had no
  `core.hooksPath` set, and the `tyt-common/build.rs` the hook header says
  activates it does not exist.
- serde_json features unify per build: `vmax-codec` already enables
  `float_roundtrip`, so any build pulling it (`vxl`, `tyt`, `tyt-vmax`) has
  the feature on. `voxj` (dev-only serde_json) and `voxj-codec`
  (`preserve_order` only) do not, which is exactly where the byte-identity
  tests run; the manifest additions make per-crate tests prove it. The
  feature earns its place: measured on serde_json 1.0.150, 19 of the 256
  linear values an 8-bit component decodes to come back from a save and
  load with different bits without it, and 0 with it. Over arbitrary
  finite `f64` the miss rate is 29.5 percent.

## The property rename

### The vocabulary source

`projects/utilities/voxsmith/src/gltf_attributes.rs` defines the voxj
property-name strings and the glTF metadata keyed on them:
`BASE_COLOR_FACTOR`, `METALLIC_FACTOR`, `ROUGHNESS_FACTOR`,
`EMISSIVE_FACTOR`, `TRANSMISSION_FACTOR` rename (values and identifiers);
`OCCLUSION_STRENGTH`, `EMISSIVE_STRENGTH`, `IOR` stay. Three name-keyed
lookups change in lockstep: `default_scalar` (L34), `scalar_range` (L49),
`default_color` (L64). `gltf_attribute_kind.rs` holds the one production
name classifier, `GltfAttributeKind::of`. Call sites of the classifier
and the three lookups (only `mesh_object.rs:192` calls the classifier;
the rest call `default_scalar`, `default_color`, or `scalar_range`):
`vxl/src/implementation/mesh_object.rs:192`,
`voxsmith/src/internal/gltf/bake_atlas.rs:124,150`,
`internal/gltf/material_document.rs:243,252`,
`convert/voxelize/voxelize_mesh.rs:482`,
`convert/vmax/from_vmax_file.rs:365`.

### glTF wire contexts that keep the Factor names

There are no glTF serde structs; the wire side is serde_json literal keys
on export and `gltf` crate accessors on import.

- `voxsmith/src/internal/gltf/material_document.rs`: the output keys
  `"transmissionFactor"` (L238), `"emissiveFactor"` (L322), plus `"ior"`
  and `"emissiveStrength"`. L233-239 pairs the voxj constant with the wire
  literal in one tuple; after the rename they are different strings, so
  the tuple must not be collapsed to one.
- `voxsmith/src/convert/gltf/from_gltf_bytes.rs`: the `gltf` crate calls
  (`base_color_factor()`, `metallic_factor()`, `roughness_factor()`,
  `emissive_factor()`, `transmission_factor()`, L249-306, L1253-1255) and
  the hand-written glTF JSON in the test GLB builders (L534-646, L805,
  L956-979) keep their names. The voxj-side assertions in the same file
  (L1099-1215, L1530-1663) rename.
- The spec's glTF conventions table keeps one glTF citation per row in its
  description column; only the property column renames.

### What renames, by file

Constant users in voxsmith (approximate hits): `voxelize_mesh.rs` 21,
`bake_atlas.rs` 27, `reduce_palette.rs` 20, `write_vmax.rs` 12,
`from_vmax_file.rs` 11, `resolve_cell_color.rs` 9 (including the failure
text at L42), `from_qbcl_file.rs` 8, `order_palette_colors.rs` 7,
`from_goxl_file.rs` 7, `from_mvox_file.rs` 6, `from_qb_file.rs` 4,
`from_qbt_file.rs` 4, `to_mvox_file.rs` 3, and the gltf test files
`material_atlas.rs`, `object_to_material_glb.rs`,
`object_to_material_gltf.rs` (3-4 each).

Constant users in vxl: `commands/mesh/texture.rs` 20 (the preset
lowering), `commands/mesh/channel_source.rs` 19 (parse-input literals in
tests), `commands/mesh/channel_packing.rs` 10, `mesh_object.rs` 8.

Bare literals (crates below voxsmith keep literals; the constants cannot
reach them): `voxcore/src/vox_palette.rs` 16 (L470-567),
`vox_main.rs` 6, `vox_property.rs` 2 (doc examples);
`voxsmith/convert/vmax/to_vmax_file.rs` 12,
`convert/voxj/from_voxj_file.rs` 11,
`internal/voxj/vox_palette_from_voxj_palette.rs` 4,
`internal/gltf/used_materials.rs` 1;
`voxj-codec/src/validate_voxj_file.rs` 8, `check_voxj_file.rs` 3,
`voxj_palette_material_counts.rs` 1; `voxj/src/voxj_file.rs` 2 (the test
fixture, L46 and L103).

vxl test fixtures: `palette_show.rs` 71 (all in `mod tests`; L834-835 is a
`--width 30` expectation whose comment counts the
`0."baseColorFactor" ` prefix, so the shorter names re-baseline it),
`palette_list.rs` 20, `info.rs` 6, `hierarchy_show.rs` 2,
`property_binding.rs` 3, `texture_map.rs` 3, `mesh.rs` 3.

CLI help prose (clap doc comments ship in `--help`):
`vxl/commands/mesh/mesh.rs` L98-102, `texture_arg.rs`, `texture.rs`,
`texture_bake.rs`, `commands/voxelize/voxelize.rs` L59,
`out_of_range_factor.rs` L4,
`commands/palette/palette_show/palette_show_label.rs` L10; voxsmith's
`convert/voxelize/out_of_range_factor.rs` L2. Internal doc prose:
`internal/mesh/mesh_material.rs` L4-26, `material_coefficient_scale.rs`,
`resolve_cell_color_or_transparent.rs`, `material_bake.rs`, the goxl and
qbcl writers, `internal/vmax/voxel_max_attributes.rs` L2-4.

Judgment item: `internal/mesh/mesh_material.rs` carries an internal
`emissive_factor` field (with `sample_material.rs:261` and
`voxelize_mesh.rs` reads). The mesh side genuinely samples factor times
texture, so the field may keep its name; decide and log.

Out of scope: treegrid's render sample data (76 hits across 6 files plus
its README) uses the old names as display strings only, and its rendering
tests pin column widths; renaming there is cosmetic and not part of this
plan. Production display code nowhere special-cases the names: the
swatch-vs-number decision is kind-driven today
(`palette_show.rs` `classify`, L290).

Spec sections carrying the names
(`projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`, 46 hits):
the kind table's typical-use column (L237-245), the palette and property
examples (L266-324), the unbound-default rule (L291), the glTF conventions
table (L351-358) and the emissive-composition paragraph (L362, which names
`emissiveFactor` and `baseColorFactor` in both the voxj and the glTF
sense), and the layer-override worked example (L525-603).

## The kind rework

### voxj

`voxj_value_pool.rs` (118 lines) is internally tagged today
(`tag = "kind"`, `rename_all = "kebab-case"`, `deny_unknown_fields`), each
variant carrying its own `values` field, `Float` and `Int` carrying
`min: VoxjBound, max: VoxjBound`. The README's target is adjacently tagged
(`content = "values"`) on bare `Vec` payloads. `rename_all` stays, but it
no longer suffices alone: serde's kebab-case inserts a separator only
before an uppercase character, never before a digit
(`serde_derive/src/internals/case.rs`), so `Vec3Float` spells itself
`vec3-float` and the six vector variants each need an explicit
`#[serde(rename = "vec-3-float")]` and kin. One method, `values_len`,
matches all eleven current arms. `voxj_bound.rs` (101 lines) hand-rolls a
serialize that writes integral values as JSON integers and a visitor that
rejects non-finite numbers; it deletes, and its visitor pattern is the
seed for the sentinel serde module. `voxj_file.rs` tests hold the only two
inline JSON kind fixtures in the repo (`document()` and `wire_document()`,
one `float` with bounds and one `srgb-hex`).

### voxj-codec

Production per-kind rules live only in
`internal/voxj_validation/check_value_pools.rs` (202 lines): the non-empty
check, then a nine-arm match over the eleven kinds (`Json`, `Bool`, and
`String` share one arm) into `check_numeric` (the bound rules),
`check_hex`/`is_hex_color` (uppercase-only spelling), and
`check_colors` (sRGB `[0, 1]`, linear `>= 0`). `validate_voxj_file.rs` and
`check_voxj_file.rs` production code is kind-agnostic; their 34 symbol
hits are tests on a shared `SrgbaHex` plus unbounded `Float` fixture.
Tests that die with the bounds and colors: the four bound tests, the two
hex tests, the two color range tests; the structural majority survives on
edited fixtures. `check_voxj_file.rs`'s doc block spells all 13 checks;
its `value-pools` item rewrites to a shape statement.

### voxcore

`vox_value_pool_kind.rs` carries four color variants (`Srgb`, `Srgba`,
`LinearRgb`, `LinearRgba`; the wire's six collapse to four here) and
bounds on `Float`/`Int`, payloads in `IdField<BVoxValuePoolValue, T>`.
`vox_value_pool_value_ref.rs` mirrors it
and is the enum every downstream match keys on. `vox_value_pool.rs` (766
lines): nine constructors (four color ones delete, `float`/`int` lose
bound parameters, six vector ones arrive), six separate kind matches
(`first_flaw`, `clone_value_pool`, `release_value_stable`, `gc_values`,
`value_ref`, `Drop`), `first_color_flaw`, and six bound free functions
(`bounds`, `bounds_well_formed`, `value_in_bounds`, `int_value_in_bounds`,
`int_at_least`, `int_at_most`). The exact-integer comparison at L475-514
with its test `int_compares_values_against_bounds_exactly` (L711) is the
repo's only 2^53 machinery in code that this change touches; the limit is
also stated in the spec, in voxj-codec's `max_hilbert_bits.rs`, and in
voxcore's `vox_value.rs`, and all three survive. Also touched:
`vox_value_pool_flaw.rs` (the `Bound` variant), `vox_bound.rs` (deletes),
`error.rs`, which carries two bound variants, not one
(`MalformedValuePoolBound` at L23 and L237 on the construction path, and
`ValuePoolBound` at L143 with its Display at L400 on the validate path,
constructed only by the `vox_main.rs` arm below), `vox_main.rs` `validate`
mapping (L1160-1170) plus 22 test-fixture hits, `vox_effective_palette.rs`
(5 test hits), `vox_effective_property.rs` (returns the ref enum).

### voxsmith

The voxj seam: `internal/voxj/vox_value_pool_from_voxj_value_pool.rs`
(read side; hex decode is `byte as f64 / 255.0`, not a transfer decode)
and `voxj_value_pool_from_vox_value_pool.rs` (write side, the densest
file: 17 `VoxjValuePool::` references, three-way `ColorFormat` dispatch
per color arm; its tests hold the repo's only independent transfer
reference implementations, L261-276, worth keeping as the seed for the
exactness test). `convert/voxj/color_format.rs` deletes along with
`VoxjFileBuilder::color_format` (`voxj_file_builder.rs`),
`to_voxj_file.rs:17`, and the `internal/voxj/write_voxj.rs` plumbing.

`internal/value_pool_color.rs`:

```rust
pub fn value_pool_color(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<[u8; 4]>
```

Seven call sites across five files: `internal/resolve_cell_color.rs:27`,
`internal/gltf/bake_atlas.rs:127`, `internal/vmax/write_vmax.rs:750,850,
1107`, `convert/mvox/to_mvox_file.rs:135`, `reduce_palette.rs:175`.

Color value pool construction sites beyond the README's list (all
compile-breaking when the color constructors delete):
`convert/goxl/from_goxl_file.rs`, `convert/mvox/from_mvox_file.rs`,
`convert/qbcl/from_qb_file.rs`, `from_qbcl_file.rs`, `from_qbt_file.rs`,
`convert/vmax/from_vmax_file.rs`, `to_vmax_file.rs`,
`convert/voxelize/voxelize_mesh.rs`, `convert/gltf/material_atlas.rs`,
`object_to_material_glb.rs`, `object_to_material_gltf.rs`,
`order_palette_colors.rs`, `internal/resolve_cell_color.rs`.
`internal/gltf/used_materials.rs` belongs to the same iteration but for
the other reason: its only value pool call is
`VoxValuePool::float(VoxBound::None, VoxBound::None, values)` at L152, so
it breaks when `float` loses its bound parameters, not when the color
constructors delete.

`convert/gltf/from_gltf_bytes.rs` L249-277 sRGB-encodes glTF's linear
factors into stored u8 on import, the inverse of the target; the linear
pivot makes it a pass-through. The destination is `MeshMaterial`, whose
`base_color` (L14) and `emissive_factor` (L24) are `TySrgbaU8`, so the
pass-through turns those fields linear, and `voxelize_mesh.rs`'s
`HashMap<[u8; 4], _>` dedup (L406-424) and `material_key` (L518-533)
follow, changing palette dedup granularity.

### The transfer

There is no color crate of our own; the transfer is the `palette` crate's
`IntoLinear`/`FromLinear`, reached through ty-math's type aliases
(`TySrgbaF64`, `TyLinSrgbaF64`, and kin). Every current 8-bit path normalizes by
255 into `f64` and then applies the analytic curve
(`into_format::<f64, f64>().into_linear()`), and that pair is already an
exact inverse over all 256 codes: measured against palette 0.7.6 with the
production spellings, u8 to linear to u8 fails on 0 of 256. palette also
ships direct `u8` conversions via lookup table (`encoding/srgb.rs`
L125-151) with upstream identity tests, but they buy nothing here and cost
two things. Their table holds f32-rounded values widened to `f64`
(`encoding/srgb/lookup_tables.rs`), so the stored linear number would
change on 254 of 256 codes by up to 1.5e-7, re-baselining every color
fixture; and their `FromLinear<f64, u8>` routes through `linear as f32`
(`encoding/srgb.rs` L146-150), which disagrees with today's `f64` encode
on 1225 of 100000 uniform linear samples, the arbitrary values the atlas
bake produces. So the boundaries keep the analytic path and the README's
exactness test lands as a regression guard. Transfer call sites:
`reduce_palette.rs:532,536`, `value_pool_color.rs:22,25`,
`sample_material.rs:243-304`,
`bake_atlas.rs:192,201`, `voxj_value_pool_from_vox_value_pool.rs:167,174`,
`write_vmax.rs:753,853`, `from_gltf_bytes.rs:252,276`, plus display-side
`vxl/palette_show.rs:420,423` and `treegrid/color/tree_grid_value.rs`.

### vxl

`utilities/voxj_color_format.rs` (23 lines) deletes with the voxj
`--color-format` flag in `utilities/voxj_encoding_options.rs:31` (accessor
L65, four tests) and its plumbing: `commands/to/voxj/to_voxj.rs`,
`commands/voxelize/voxelize.rs`, `dependencies.rs`,
`implementation/dependencies_impl.rs`, `implementation/to_voxj.rs`,
`implementation/voxelize.rs`, `implementation/write_voxj_document.rs`,
`utilities/mod.rs`. The vmax `--color-format`
(`commands/to/vmax/color_format.rs`, `png`/`plist`/`all`, plus tyt-vmax's
twin) is an unrelated flag and stays.

`implementation/mesh_object.rs`: `value_pool_kind` (L213-228) maps color
kinds to `ChannelKind::Color` and scalars to `Scalar`; its caller
`channel_kind` (L187-209) already falls back to `GltfAttributeKind::of`
for the unbound case, so the rework promotes the name path.

`implementation/palette_show.rs`: `fn classify(&VoxValuePool) -> Kind`
(L290-313) plus seven kind-keyed helpers (`sample` L317, `sample_color`
L337, `sample_number` L375, `sample_other` L391, `color_bytes` L411,
`color_floats` L433, `alpha_component` L447), four of them carrying
`// classify() routes only ...` comments the name-keyed rework
invalidates; the `classify` call
site inside `build_collection` (L219) already has the property key in
scope. Fixture weight: 21 code hits here, 3 in tests; `palette_list.rs`,
`info.rs`, and `hierarchy_show.rs` carry small fixture counts.

### Census

Old kind strings under `projects/`: the spec 44, `voxj_color_format.rs` 6,
`color_format.rs` 6, `voxj_value_pool_from_vox_value_pool.rs` 4,
`from_voxj_file.rs` 3, `validate_voxj_file.rs` 2, `voxj_file.rs` 1,
`check_voxj_file.rs` 1. Open plan pages:
`vxl-commands/reference/palette/remap.md` 2. The asset carries 2.

Identifiers: the six voxj color variants, 46 occurrences under
`projects/` (per-file table in the sections above); `VoxjBound` 74;
`VoxBound` 144, 75 of them in voxcore.

## Assets

Exactly one on-disk voxj file exists:
`submodules/tyt-assets/scratch/energy-turret.voxj`, regenerated at
closeout (2026-08-05) from its sibling `energy-turret.glb` by the default
`vxl voxelize` invocation, whose one-voxel-per-meter grid matches the
asset's 16x13x11 bounds. It carries one `vec-4-float` pool, one
`vec-3-float`, six plain `float`s, and the renamed property names. The
survey's claim that regeneration takes a tyt-assets commit and a gitlink
bump was wrong: the submodule's `.gitignore` covers `/scratch/`, the file
is untracked, and the overwrite touches no commit anywhere. Nothing in
the workspace reads the file. No `.voxjz` exists anywhere.

## Corrections to the README's blast radius

The README expects the implementation to surface more; these five
surfaced in this survey and the checklist carries them:

1. The int cap and spelling rules are new in the parse, not moved from
   voxj-codec validation, whose numeric check compares in `f64` and
   carries no magnitude cap; serde's typed parse already rejects
   fractional and exponent spellings implicitly. The only exact-integer
   machinery is voxcore's bound comparison, which deletes with the
   bounds. The 2^53 limit is stated elsewhere (the spec, voxj-codec's
   `max_hilbert_bits.rs`, voxcore's `vox_value.rs`), and those stay.
2. voxcore's `error.rs` (both bound variants, `MalformedValuePoolBound`
   and `ValuePoolBound`) and `vox_effective_palette.rs` are in the blast
   radius. `vox_effective_property.rs` only returns the ref enum and
   compiles through the rework unchanged.
3. Roughly twenty more voxsmith and vxl files construct color value pools
   through the voxcore constructors (the list above); they break at
   compile, not just at test churn.
4. The u8 identity test passes on today's normalize-then-curve path, which
   is already an exact inverse pair over all 256 codes, so it adds a
   guard and forces no transfer change. Moving to palette's lookup tables
   would be a regression: it shifts the stored linear value on 254 of 256
   codes and encodes arbitrary linear values through `f32`.
5. voxj's serde_json is a dev-dependency, so its `float_roundtrip`
   addition lands under `[dev-dependencies]`; and feature unification
   means only per-crate test runs prove the voxj-codec addition. The
   README's premise for the feature is measured, not assumed: without it
   19 of 256 decoded linear values change bits across a save and load.
