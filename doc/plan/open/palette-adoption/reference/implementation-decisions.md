# Implementation decisions

Non-obvious code-level choices made while executing the
[checklist](../checklist.md). Append as sessions land; read before editing to
stay consistent. Facts verified at the keyboard, not just planned.

## S1 - palette dependency (2026-07-22)

- **Version bump 0.1.8 -> 0.1.9, NOT 0.2.0.** The plan calls the change
  "breaking", but this repo keeps ty-math in the `0.1.x` line and bumps the
  patch component across breaking churn (0.1.7 -> 0.1.8 was itself breaking, the
  treegrid/voxj adoption). Every consumer pins a `^0.1.x` caret requirement
  (`0.1.0` / `0.1.7` / `0.1.8`, all `>=x, <0.2.0`), resolved through
  `[patch.crates-io]` to the path crate. A `0.2.0` bump satisfies none of those
  carets, so the patch would go unused and workspace resolution would fail -
  breaking even `cargo check -p ty-math`. Staying at 0.1.9 keeps every caret
  satisfied. This also matches S1's stated scope (workspace + ty-math Cargo.toml
  only, no consumer Cargo.toml edits).
- **Workspace Cargo.toml edit is a no-op here.** The plan's S1 names the
  workspace `Cargo.toml`, but this workspace has no `[workspace.dependencies]`
  table (only `members`, `resolver`, and `[patch.crates-io]` path overrides).
  palette is an external crate with no path override, so it is added ONLY to
  `ty-math/Cargo.toml`, not patched.
- **palette resolved to 0.7.6**, matching the api-map's verified version.
  Transitive deps pulled: `approx` 0.5.1, `by_address` 1.2.1, `fast-srgb8`
  1.0.0, `palette_derive` 0.7.6.
- **`approx` feature left OFF for now.** Added only `default-features = false` +
  `["std"]`; `serializing` / `named` / `random` stay off. `approx` is a
  transitive dep but not a palette feature yet; enable palette's `approx`
  feature in S2 only if the ported ty-math color tests want approx asserts.
- Gate met: `cargo check -p ty-math`, `cargo fmt --all --check`, and
  `cargo clippy -p ty-math --all-targets -- -D warnings` all green (the dep is
  unused, and `unused_crate_dependencies` is allow-by-default, so no warning).

## S2 - alias flip + glue (2026-07-22)

The atomic step. ty-math now compiles standalone (78 tests green); the workspace
is RED until S8.

- **The `*_u8 / *_f32 / *_f64` alias files needed NO edits.** They already read
  `pub type TySrgbaU8 = TySrgba<u8>;` over the base tyt alias, so re-pointing the
  base type to palette retargets them transitively (`TySrgbaU8` is now
  `Srgba<u8>`). Left all fourteen untouched.
- **`TyOklabColorF64` / `TyCielabColorF64` already existed** (the ty-color-model
  plan shipped the full f32/f64 family), so there was nothing to "add" -
  `TyCielabColorF64 = TyCielabColor<f64> = Laba<D65, f64>` already resolves
  correctly. The checklist's "ADD" wording is stale; both aliases were in place.
- **`TyHexColor` re-implements tyt's exact byte parsing, does NOT wrap palette's
  `FromStr`.** palette's `Srgba<u8>: FromStr` rejects 6-digit, accepts 3/4-digit
  shorthand, and returns `Result` - incompatible with tyt's 6-or-8 + opaque
  default + `Option` contract. Porting the original parse verbatim is the
  faithful glue. `to_hex` reads palette fields `.red/.green/.blue/.alpha`.
- **`TyColorToVector3` reads palette FIELDS, not `into_components()`.** The
  api-map suggested `into_components()`, but plain field access is simpler and
  identical: `self.red/.green/.blue` for `TySrgb`, and `self.l/.a/.b` for the
  Alpha-wrapped Oklab/Lab (field access auto-derefs `Alpha` to its `.color`).
  Alpha is dropped by simply not reading it.
- **Friction 3 RESOLVED at the keyboard - no drift.** `into_format::<u8>()` on an
  exact `0.5` yields `128`, matching tyt's `to_unorm8` (round-half-away-from-zero
  + clamp). The round-trip and clamp tests pass byte-exact; NO golden was
  re-baselined for rounding, and none is expected downstream on sRGB paths.
- **Friction 6 confirmed.** Both scalar `c * k` and componentwise `a * b` scale
  alpha, matching tyt's `Mul<T>` / `componentwise_multiply`. `into_linear` /
  `from_linear` exist and round-trip; `.color` is the drop-alpha.
- **Deleted tests (behaviors abandoned per plan):**
  `to_srgba_sign_extends_out_of_gamut` (was in ty_lin_srgba) and
  `u8_color_keys_a_hash_set` (was in ty_srgb and ty_srgba). The out-of-gamut
  sign-extension and the color `Hash` are both gone.
- **Friction 4 (Default alpha flips to opaque): not exercised by ty-math.** No
  ty-math test calls a color `::default()`, so nothing re-baselined here; the
  consumer re-audit stays for Phase 2. See [[palette-adoption-plan]].
- **lib.rs re-exports** consolidated into one nested use:
  `pub use palette::{FromColor, IntoColor, white_point::D65};`. Deleted
  `srgb_transfer` (mod + file + `pub(crate) use`); the `ty_array_conversions!`
  macro STAYS (vector2/3/4 and quaternion still call it).

## S3 - voxsmith/convert, 9 files (2026-07-22)

All 9 convert files migrated. voxsmith lib still RED (S4 files remain), so its
tests could not run yet - verified instead by ZERO convert-file errors under the
full feature superset.

- **Conversions applied:** `into_format::<f64>()` (Srgb, one param) /
  `::<f64, f64>()` (Srgba, two params) / `::<u8, u8>()`; `from_array` ->
  `Ty::from`; `to_array` -> explicit `<[T; N]>::from(..)` inside closures / map
  keys, `.into()` at return position; `to_srgb` DROP-ALPHA -> `.color`; the gltf
  encode `to_srgba().to_u8()` -> `TySrgbaF64::from_linear(lin).into_format::<u8,
  u8>()`; hex -> `TyHexColor` (import added to 5 test modules + the from_gltf
  test module); `const DEFAULT_FILL: TySrgbaU8 { .. }` -> `const DEFAULT_FILL:
  [u8; 4]` with `TySrgbaU8::from(DEFAULT_FILL)` at its 3 use sites.
- **LESSON - verify voxsmith under `--features gltf`, not just default.** The
  `gltf` / `_mesh` convert paths are feature-gated and OFF by default
  (`default = goxl,mvox,qbcl,vmax,voxj`), so a plain `cargo check -p voxsmith`
  never compiles `convert/gltf/from_gltf_bytes.rs` or the mesh code in
  `voxelize_mesh.rs`. Three real `.r/.g/.b` + encode sites lived there and only
  surfaced under `--features gltf`.
- **Sites the type-name grep MISSED** (they name no color type, operating on a
  helper return or a struct field) - all under the gltf superset:
  `from_gltf_bytes.rs:249` `let base_color = linear_rgba(..).to_srgba().to_u8()`
  (same encode as `emissive_srgb`); `voxelize_mesh.rs` `material_key`
  `material.base_color.to_array()` -> `<[u8; 4]>::from(material.base_color)` and
  `[emissive.r, emissive.g, emissive.b]` -> `.red/.green/.blue`. Rely on the
  compiler under the superset, not just a grep.
- **Left foreign `.r/.g/.b/.a` alone:** `GoxlVoxel` (goxl 71/126), `MVoxColor`
  (mvox 102), and the `TyVector3` `to_f64` / `from_array` in from_vmax.
- **`.into()` vs explicit `<[T; N]>::from`:** used `.into()` where the target is
  unambiguous (function return position: the three `color_floats`, the test hex
  helpers) and explicit `<[T; N]>::from(..)` inside `.map()` closures, `let [..]`
  destructures, and `HashMap::entry` keys, to avoid leaning on cross-expression
  inference. Both compile; explicit is clearer at the ambiguous sites.
- **Verification gap:** voxsmith's own tests (`cargo test -p voxsmith`) run at S4
  once the lib compiles. S3 convert test-module edits (hex helpers, `.red`
  renames) typecheck as far as rustc's error recovery reaches during the failed
  build; final confirmation is S4's gate.

## S4 - voxsmith/internal + reduce_palette, 8 files (2026-07-22)

voxsmith now fully GREEN: `cargo test -p voxsmith` 141 passed; `--features gltf`
212 passed; clippy clean under both. This also confirms the S3 convert test
modules (they run inside these totals). Two files needed no edits (mesh_material,
mesh_base_color_map - pure alias fields).

- **NEW FRICTION - `into_color::<T>()` does NOT compile.** The api-map's
  `.into_color::<TyOklabColorF64>()` is wrong: palette's `into_color` is
  `IntoColor<T>::into_color(self) -> T`, so the target is a TRAIT parameter, not
  a method generic (rustc E0107, "method takes 0 generic arguments but 1
  supplied"). RESOLUTION: use `FromColor::from_color` -
  `TyOklabColorF64::from_color(linear)` / `TyCielabColorF64::from_color(linear)`
  - with the source `linear` type pinned (equivalently a typed
  `let x: TyOklabColorF64 = linear.into_color();`). Only reduce_palette's
  `to_space` needs it (the sole color-space converter; census confirms Oklab/Lab
  are reached nowhere else). ty-math already re-exports `FromColor`.
- **NEW FRICTION - `into_linear()` on `Alpha` is generic over its output
  component type(s)** and errors E0283 ("type annotations needed") when the
  downstream context does not pin it. RESOLUTION: annotate the result,
  `let linear: TyLinSrgbaF64 = <srgba>.into_linear();`, at each decode site
  (voxj `decode_rgb`/`decode_rgba`, write_vmax x2, bake_atlas
  `emissive_color_bytes`, sample_material `emissive`). Where a same-type Mul
  already pins it (sample_material `base_color_linear`, `... .into_linear() *
  map.factor`), no annotation is needed. Added `TyLinSrgbaF64` imports to voxj
  and write_vmax for the annotations.
- **Collapsed the three identical 4-arm pool decodes into `pool_color`** (the
  sanctioned optional DRY win). `material_color` (reduce_palette) ->
  `pool_color(pool, value_id)`; `color_bytes_or` (bake_atlas) ->
  `value.and_then(|(pool, id)| pool_color(pool, id)).unwrap_or(default)`.
  Behavior is identical: a color pool with an out-of-range id yields `None` ->
  `default`, exactly as the old per-arm `.unwrap_or(default)`. Removed ~50 lines
  of duplicated match arms. `pool_color` is `pub(crate)` (reachable crate-wide
  via `internal::*`).
- **Encode:** the linear->sRGB byte encode `to_srgba().to_u8()` becomes
  `TySrgbaF64::from_linear(lin).into_format::<u8, u8>()` (pool_color Linear arms,
  sample_material base/emissive, bake_atlas emissive).
- **`componentwise_multiply(&map.factor)` -> `* map.factor`** (by-value Mul,
  Hadamard including alpha) in sample_material.
- **Field renames** `.r/.g/.b/.a` -> `.red/.green/.blue/.alpha` on tyt linear
  colors (sample_material, write_vmax, voxj, bake_atlas). Dropped two stale doc
  links to the removed `TySrgba::to_lin_srgba` in voxj.
- **Friction 2 (CIELAB drift) caused NO test churn.** `reduces_across_all_color_
  spaces` and both dither-golden tests pass unchanged - the clustering asserts on
  representative selection, not exact Lab values, and the sRGB dither coords are
  `byte/255` (into_format-exact). No golden re-baselined in S4.
- **reduce_palette imports** net: added `FromColor`, `TyCielabColorF64`,
  `TyColorToVector3`, `TyOklabColorF64`; kept `TyLinSrgbaF64`, `TySrgbaU8`,
  `TyVector3F64`, `TyVector3U32`; dropped `TySrgbaF64` and voxcore `VoxValuePool`
  (both unused after the collapse).

## S5 - vxl, 1 file (2026-07-22)

vxl GREEN standalone: `cargo test -p vxl` 228 passed; check/clippy clean (lib +
`--features bin`).

- **vxl reaches green WITHOUT S6.** vxl depends on treegrid but enables only
  `features = ["json"]`, NOT `treegrid/ty-math`, so treegrid's (still-red) color
  module is not compiled into vxl. palette_show decodes colors itself. So the
  S5-before-S6 order in the checklist is fine; the crates are not coupled through
  treegrid's color feature.
- **The census's S5 notes are STALE.** The recent treegrid-adoption commits
  (after the 2026-07-22 census snapshot) refactored `srgb_hex` and `scalar_level`
  out of palette_show; `TyFloatExt` is no longer imported there. The ONLY color
  code left in all of vxl is the 4-arm `color_bytes` decode. No use-line split
  was needed.
- **Migrated `color_bytes` identically to S4's `pool_color`**: sRGB arms ->
  `<[u8; 4]>::from(TySrgbaF64::new(..).into_format::<u8, u8>())`; Linear arms ->
  `<[u8; 4]>::from(TySrgbaF64::from_linear(TyLinSrgbaF64::new(..)).into_format::
  <u8, u8>())`. Import line unchanged (both aliases still used).

## S6 - treegrid, 2 files (2026-07-23)

treegrid GREEN: default `cargo test -p treegrid` 111; `--all-features` 145 (incl.
10 color tests); clippy clean (all-features). Color module is behind the
`ty-math` feature, so it is verified with `--all-features`, not the default gate.

- **KEPT the `TyFloatExt` bound - diverges from the plan's "likely DROPS".** The
  plan asked to re-examine it; the keyboard finding is that it is LOAD-BEARING.
  Sibling `srgb8` / `srgba8` byte ctors exist (`value/tree_grid_value.rs`), and
  the bound is the type-level guard that keeps `T` to f32/f64 so 8-bit colors go
  through those byte ctors, not the float ctors. Dropping it would make
  `TreeGridValue::srgb::<u8>(..)` compile (u8: Copy+Display+Into<f64>) and blur
  the deliberate float/byte split. The documenting comment stays accurate. Import
  and bounds unchanged.
- **palette's `new` / `from_linear` / `into_format` are all component-generic, so
  the widened f64 type must be PINNED.** The old `.to_u8()` was defined only on
  the f64 instantiation, which pinned it; palette's versions are not, giving
  E0283 (same family as the S4 `into_linear` friction). Pinned via
  `TySrgb::<f64>::new(..)` and `TySrgb::<f64>::from_linear(..)` (turbofish on the
  alias). srgb/srgba only needed `::<f64>::new`; lin_rgb/lin_rgba also needed
  `::<f64>::from_linear` (the from_linear output is generic too).
- **Conversions:** `to_array` -> `<[T; N]>::from(color)`; `to_u8` ->
  `into_format::<u8>()` (3-comp) / `::<u8, u8>()` (4-comp); `to_srgb` DROP-ALPHA
  (srgba, lin_rgba) -> `.color`; `to_srgb`/`to_srgba` TRANSFER (lin_rgb, lin_rgba)
  -> `TySrgb::<f64>::from_linear`; the swatch `to_array` -> `.into()` (target
  `[u8; 3]` pinned by `TreeGridSwatch::Color`).
- **Byte-exact swatch goldens ALL preserved, no re-baseline.** `lin_rgb`
  linear 0.5 -> 188, `lin_rgba` HDR 2.0 -> clamp 255, f32 -> [64, 128, 255], the
  drop-alpha swatches -> [255, 0, 128]. palette's transfer + `into_format`
  reproduce tyt's bytes for these in-gamut / clamped cases.

## S7 - tyt-fbx + tyt-injection + ty-math-serde, 5 files (2026-07-23)

Gate green: `cargo test -p tyt-fbx -p tyt-injection -p ty-math-serde` all pass.

- **The census was exact: only ONE code line group changed.** The DTO
  `From<TySrgba> for TySrgbaSerde` body switched `c.r/.g/.b/.a` ->
  `c.red/.green/.blue/.alpha` (ty_srgba_serde:16-19). The reverse
  `From<TySrgbaSerde> for TySrgba` keeps `TySrgba::new(c.r, c.g, c.b, c.a)`
  unchanged - `.new` is palette's `Srgba::new(red, green, blue, alpha)`, and the
  DTO's own `r/g/b/a` fields are untouched (they are the wire).
- **Read palette FIELDS, not `into_components()`** (same call the api-map floated
  as an alternative), matching the `TyColorToVector3` glue decision in S2. Field
  access auto-derefs `Alpha` to the color part; `alpha` is the direct field.
- **The other 4 files compiled with ZERO edits, as predicted:**
  `create_point_cloud.rs` `TySrgba::new(r,g,b,a)` (palette `Srgba::new` takes the
  same 4 args); both fbx `Dependencies` `&[Vec<TySrgba>]` signatures (transparent
  alias); `serialize_points_and_colors_json.rs` `.copied()` (palette `Srgba<f32>`
  is `Copy`) and its `TySrgba::new` test literal.
- **Wire byte-identical.** The pinned `serializes_to_stable_json` (tyt-injection)
  passes verbatim: keys stay `r/g/b/a` because serde keys off the DTO field
  names, and the DTO is untouched. palette's `serializing` feature stays OFF.

## S8 - workspace green + audit (2026-07-23)

The migration is fully GREEN. No design change; this step is verification of S7's
completion plus the whole-tree audit.

- **Green at BOTH feature sets.** `cargo clippy --workspace --all-targets
  -- -D warnings` clean at default and at `--all-features` (the latter compiles
  the gated color paths: voxsmith `gltf`, treegrid `ty-math`). `cargo test
  --workspace` 1002 passed / 0 failed default, 1012 / 0 at `--all-features` (the
  +10 is treegrid's color tests behind `ty-math`). Per-crate gates re-confirmed
  and match S3-S6: voxsmith 141 (default) / 212 (`--features gltf`), treegrid 145
  (`--all-features`), vxl 228. `cargo fmt --all --check` clean.
- **No external wire moved - proven, not asserted.** The ONLY working-tree
  changes across the whole migration are `.rs` source, `Cargo.lock`/`Cargo.toml`,
  and the plan docs; NO golden/fixture data file is modified (`git status
  --porcelain` filtered). So no golden was re-baselined: the fbx JSON (pinned by
  the S7 test) and the vmax/voxj/goxl/qb pool bytes/hex (which cross as raw
  arrays/hex, per-crate tests unchanged) are all byte-identical. The only test
  DELETIONS remain the intended internal two from S2
  (`to_srgba_sign_extends_out_of_gamut`, `u8_color_keys_a_hash_set`); no Lab
  re-baseline and no `into_format` LSB drift ever surfaced.
- **`palette` crate named ONLY in ty-math.** Grep for `palette::` outside ty-math
  hits only the DOMAIN voxel-palette modules (mvox/voxj/vox `*_palette`,
  voxcore `b_vox_palette`, `reduce_palette`, the vxl `palette` command's local
  `mod palette`) - none name the external crate. `palette` appears in exactly one
  `Cargo.toml` (`ty-math`). The collision-free outcome the plan wanted.
- **Fixed one stale doc ref S4 missed.** A `///` on the voxj test helper
  `srgb_to_linear` still pointed at the removed `TySrgba::to_lin_srgba`; retargeted
  to `TySrgba::into_linear` (voxj_value_pool_from_vox_value_pool:212). It was a
  code-span, not an intra-doc link, so it never broke the build - but S8 mandates
  no dangling reference to a removed method. All removed-method CALLS are already
  compiler-guaranteed absent (the workspace compiles).
- **Only S9 (the single commit) remains**, gated on explicit owner approval.
