# palette adoption checklist

Ordered migration for the [plan](README.md): replace ty-math's hand-rolled color
types with `type Ty... = palette::...` aliases and move every consumer onto
palette's own API. Line numbers are from the census snapshot
([reference/consumer-census.md](reference/consumer-census.md)); confirm at the
keyboard. The alias flip is atomic (see README "Commit strategy"), so the
workspace goes red at S2 and returns to green only at S8 - stage across sessions,
commit once.

## Ground rules

- Prefer palette's behavior; do NOT preserve byte-exactness. Re-baseline internal
  tests/goldens that legitimately shift (owner relaxed this). Stop only if an
  EXTERNAL wire contract would move - none should (see S8).
- Only ty-math names the `palette` crate. Consumers import `ty_math::...` only;
  if a consumer needs `palette::`, ty-math is missing a re-export - add it.
- Repo style: Rust edition 2024, consolidated nested `use`, one public item per
  file in snake_case, one trait per extension file, doc comments on public items,
  80-column ASCII comments. `cargo fmt --all` + `cargo clippy --workspace
  --all-targets -- -D warnings` + `cargo test` before staging; the pre-commit
  hook enforces fmt + clippy.
- Verify sub-parts with `cargo check -p ty-math` (S1-S2), then per consumer crate
  (S3-S7), then `cargo check --workspace` (S8).

## Phase 1: ty-math foundation

- [ ] **S1. Add the `palette` dependency.** Workspace `Cargo.toml` +
      `ty-math/Cargo.toml`: `palette = { version = "0.7", default-features =
      false, features = ["std"] }` (+ `"approx"` only if ty-math tests want it).
      Do NOT enable `serializing` / `named` / `random`. Bump ty-math's version
      (breaking). `cargo check -p ty-math` still green (dep unused yet).

- [ ] **S2. Flip the color types to aliases + add glue.** The atomic step; ty-math
      compiles on its own, consumers break until Phase 2.
      - Replace the six base-type files with one-line `pub type` aliases
        ([api-map](reference/palette-api-map.md)): `ty_srgb`, `ty_srgba`,
        `ty_lin_srgb`, `ty_lin_srgba`, `ty_oklab_color`, `ty_cielab_color`. Pin
        `D65` on the Lab alias. Keep the doc comments.
      - Retarget the `*_u8 / *_f32 / *_f64` alias files to the palette types; ADD
        `TyOklabColorF64` and `TyCielabColorF64` (= `Laba<D65, f64>`).
      - Delete `srgb_transfer.rs` and its re-export. Remove the color uses of
        `ty_array_conversions!` (the MACRO STAYS - quaternion/vector2/3/4 use it).
      - Drop the manual `impl Eq` / `impl Hash` / `to_srgb` / `to_f64` / `to_u8` /
        `to_lin_srgba` / `to_srgba` / `to_oklab` / `to_cielab` /
        `componentwise_multiply` / `Mul<T>` / `from_hex` / `to_hex` /
        `to_vector3` / array methods (all now palette or glue).
      - Add glue, one trait per file: `TyHexColor` (`from_hex`/`to_hex` over
        palette `FromStr`/`UpperHex`, preserving `Option` + 6-or-8-digit + opaque
        default + `#RRGGBBAA`); `TyColorToVector3` (`to_vector3` for `TySrgb`,
        `TyOklabColor`, `TyCielabColor` -> `TyVector3<T>` via `into_components`).
      - lib.rs: update mods/re-exports; add `pub use palette::{FromColor,
        IntoColor};` and `pub use palette::white_point::D65;`.
      - Port ty-math's own color unit tests: keep hex/array/mul/round-trip
        coverage against the new API; DELETE the out-of-gamut sign-extension
        tests and `u8_color_keys_a_hash_set` (behaviors abandoned).
      - Gate: `cargo test -p ty-math` green; `cargo test -p ty-math-serde` after
        the DTO body change (may defer to S7).

## Phase 2: migrate consumers (workspace red until S8)

- [ ] **S3. voxsmith/convert (9 files).** `to_f64`/`to_u8` -> `into_format`
      (Srgba: two params `::<f64,f64>` / `::<u8,u8>`; Srgb: one); `to_array`/
      `from_array` -> `.into()` / `Ty...::from`; `to_srgb` DROP-ALPHA -> `.color`;
      the gltf `to_srgba().to_u8()` encode -> `from_linear(..).into_format`; hex
      -> `TyHexColor`; `const DEFAULT_FILL` -> `[u8; 4]` (voxelize_mesh:16).
      Leave every foreign `.r/.g/.b` (`GoxlVoxel`, `MVoxColor`, `Qb*/Qbcl*/Qbt*`)
      and the `TyVector3` `componentwise_multiply`/`to_f64` in from_vmax alone.
      Re-baseline the `to_hex` string asserts only if the `#RRGGBBAA` glue changed
      them (it should not). Gate: `cargo test -p voxsmith` green.

- [ ] **S4. voxsmith/internal + reduce_palette (8 files).** Base-color decode ->
      `.into_linear()`; encode -> `Ty...::from_linear(..).into_format::<u8,u8>()`;
      `componentwise_multiply(&o)` -> `o` by value `*`; `.r/.g/.b/.a` reads ->
      `.red/.green/.blue/.alpha`; `to_oklab`/`to_cielab` ->
      `.into_color::<TyOklabColorF64>()` / `::<TyCielabColorF64>()`; `to_vector3`
      -> `TyColorToVector3`. Optionally collapse the three identical 4-arm pool
      decodes (`pool_color`, `bake_atlas color_bytes_or`, `reduce_palette
      material_color`) into one helper. Keep `TyFloatExt::to_unorm8` (scalar).
      Re-baseline Lab-space clustering tests if they shift (Oklab/sRGB should
      not). Gate: `cargo test -p voxsmith` green.

- [ ] **S5. vxl (1 file).** `palette_show` 4-arm pool decode -> palette (mirrors
      S4). Leave `srgb_hex` (hand-rolled from `[u8;4]`) and `scalar_level`
      (scalar `to_unorm8`) alone; split the `use` line so `TyFloatExt` stays.
      Gate: `cargo test -p vxl` green.

- [ ] **S6. treegrid (2 files).** `tree_grid_value` + `tree_grid_json_value`
      ctors: `to_array` -> `.into()`; the f64-widened `to_u8` -> `into_format`;
      `to_srgb`/`to_srgba` DROP-ALPHA vs TRANSFER rewritten by hand (api-map);
      re-examine the generic bounds - the ctors already widen to f64 before
      converting, so `TyFloatExt` likely DROPS (net simpler). Gate: `cargo test -p
      treegrid` green.

- [ ] **S7. tyt-fbx + tyt-injection + ty-math-serde (5 files).** The DTO body:
      `From<TySrgba>` reads `c.red/.green/.blue/.alpha` (ty_srgba_serde:18). fbx
      `TySrgba::new` and `&[Vec<TySrgba>]` signatures compile unchanged.
      Confirm the pinned `r/g/b/a` JSON test stays byte-identical. Gate: `cargo
      test -p tyt-fbx -p tyt-injection -p ty-math-serde` green.

## Phase 3: sweep, verify, commit

- [ ] **S8. Workspace green + re-baseline.** `cargo check --workspace`, `cargo fmt
      --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test
      --workspace`. Confirm the only test changes are the intended ones (dropped
      out-of-gamut/Hash tests, any Lab-drift re-baseline, `into_format` +/-1 LSB
      if it surfaced). Confirm NO external wire moved: fbx JSON, and vmax/voxj/
      goxl/qb pool bytes/hex, all identical. Grep that no consumer names
      `palette::` and no dangling reference to a removed method/type remains.

- [ ] **S9. One clean commit.** Stage everything (code + these checklist ticks).
      Present the staged diff for owner review; commit once, directly on main,
      with a Conventional Commits subject and the `Co-Authored-By: Claude Fable 5
      <noreply@anthropic.com>` trailer, only on explicit approval.

Gate (whole plan): workspace green, clippy clean, every external wire identical,
`palette` named only inside ty-math, ty-math no longer maintains its own color
math (transfer, conversions, arithmetic, arrays, Eq/Hash) - only the two glue
bridges (`TyHexColor`, `TyColorToVector3`) and the serde DTO remain hand-written.
