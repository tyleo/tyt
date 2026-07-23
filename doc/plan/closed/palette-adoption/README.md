# Back ty-math's color types with the `palette` crate

Status: **closed.** Landed as one commit (`e54da22`, 2026-07-23), rebased on
origin's latest. All nine steps (S1-S9) shipped: ty-math's color types are now
`type Ty... = palette::...` aliases, every consumer moved onto palette's own
methods, and the workspace is green (1006 tests default / 1016 `--all-features`,
clippy clean) with no external wire moved. The per-step keyboard record is in
[reference/implementation-decisions.md](reference/implementation-decisions.md).
A direct follow-up to the closed
[ty-color-model plan](../ty-color-model/README.md),
which built palette-STYLE color types by hand (`TySrgba<T>` / `TyLinSrgba<T>` /
`TySrgb<T>` / `TyOklabColor` / `TyCielabColor`, the color space as the type
identity and the component as a generic). This plan replaces those hand-rolled
types with thin `type Ty... = palette::...` aliases so ty-math re-exports the
real `palette` crate under the tyt names, and migrates every consumer onto
palette's own methods.

## Goal in one paragraph

The hand-rolled types already have palette's SHAPE; they should have palette's
CODE. Two wins: (1) take palette's implementations when they are better
(reference-correct Oklab/CIELAB, a maintained transfer function, hex, arithmetic,
array casts); (2) stop maintaining our own color math. The tyt names stay the
public vocabulary - consumers keep writing `TySrgba`, never `palette::` - so
`palette` is an implementation detail confined to ty-math. This confinement also
keeps the domain "palette" concept (voxcore `BVoxPalette`, `reduce_palette`,
`palette_show`) collision-free, since only ty-math ever names the crate.

## What "palette doesn't leak" means here (owner, 2026-07-22)

A `type` alias is transparent, so palette's field names (`red/green/blue/alpha`),
method names (`into_format`, `into_linear`, `into_color`), and helper types DO
become the vocabulary consumers write. The owner has explicitly waived the
field-rename concern. "Doesn't leak" therefore means the narrow, achievable
thing: **no consumer crate names the `palette` crate.** ty-math re-exports every
alias plus the `IntoColor` / `FromColor` traits and `D65`, so a consumer's only
color imports are `ty_math::...`. That is the contract this plan holds.

## Decisions

- **Aliases (verified).** The eight types map cleanly; see
  [reference/palette-api-map.md](reference/palette-api-map.md). `TyCielabColor`
  MUST pin the white point (`palette::Laba<D65, T>`) because palette puts `Wp`
  first. Add f64 aliases for Oklab/Lab (palette defaults them to f32; every tyt
  conversion runs on f64).
- **Adopt palette's behavior; do not preserve byte-exactness (owner relaxed).**
  Where palette differs, prefer palette and re-baseline tests. Only an EXTERNAL
  wire contract needs sign-off before it moves - and none does here (see Blast
  radius). Internal golden churn is a normal part of a step, not a stop.
- **Keep two thin glue helpers, both bridges rather than color math.**
  `TyHexColor` (`from_hex`/`to_hex`, preserving tyt's `Option` + 6-or-8-digit +
  `#RRGGBBAA` contract over palette's `FromStr`/`UpperHex`), and
  `TyColorToVector3` (`to_vector3`, since palette cannot know ty-math
  `TyVector3`). Everything else moves to palette's own API.
- **Keep the serde DTO.** `TySrgbaSerde` (`r/g/b/a` f32) stays hand-rolled and
  palette's `serializing` feature stays off; only the `From<TySrgba>` body reads
  palette accessors. The fbx point-cloud JSON is byte-identical.
- **Drop tyt's manual `Eq`/`Hash`/`Default`/transfer/array code.** palette
  provides `Eq` (u8) / `PartialEq` (float) identically; `Hash`/`Ord` are not
  needed (every dedup already keys on `[u8; N]`); the sRGB transfer and array
  conversions come from palette. The `ty_array_conversions!` macro STAYS for the
  vector/quaternion types that still use it.

## Friction (eyes-open costs)

1. **Out-of-gamut transfer differs.** palette applies the sRGB formula directly
   (no CSS Color 4 sign extension); a negative component NaNs on the power
   branch. In-gamut `[0, 1]` is identical, positive HDR is fine (clamps at u8),
   and negative linear components do not occur on production paths. tyt's
   out-of-gamut sign-extension UNIT TESTS are dropped (that behavior is
   abandoned).
2. **CIELAB drifts ~2-3 sig figs.** palette's D65 constants and primaries-derived
   matrix differ from tyt's hardcoded ones. Re-baseline any exact Lab assertion.
   Oklab is byte-identical (same fused matrix), so Oklab-space clustering is
   unaffected. The sRGB-space dither tests are byte-exact and safe (coords are
   `byte/255`, which `into_format` reproduces exactly).
3. **`into_format::<u8>()` rounding** vs `TyFloatExt::to_unorm8`
   (round-half-away-from-zero + clamp): expected to match, at most +/-1 LSB on an
   exact `.5`. Verify at the keyboard; accept and re-baseline if it drifts.
4. **`Default` alpha flips to opaque** on palette (`Srgba::default().alpha ==
   max`, tyt's derive gave 0). Audited clear today (no consumer calls a color
   `::default()`; no consumer struct derives `Default` with a color field);
   re-audit at the keyboard.
5. **Const color literal.** `const DEFAULT_FILL: TySrgbaU8 = TySrgbaU8 { .. }`
   (voxelize_mesh:16) has no const palette form; becomes `const DEFAULT_FILL:
   [u8; 4]` constructed via `TySrgbaU8::from(..)` at its 3 use sites.
6. **`to_srgb` is overloaded** in consumers: DROP-ALPHA (`TySrgba<u8> ->
   TySrgb<u8>`, becomes `.color`) vs TRANSFER (`TyLinSrgb<f64> -> TySrgb<f64>`,
   becomes `into_color`/`into_encoding`). Rewrite by hand, never find-replace.
   `componentwise_multiply(&o)` becomes the by-value `*` operator.

## Blast radius

ty-math color module (6 base types + 3 alias families -> aliases, plus glue and
re-exports; delete `srgb_transfer.rs` and the color uses of the array macro), one
DTO body in ty-math-serde, and ~24 consumer files across voxsmith (convert +
internal + reduce_palette), vxl, treegrid, and tyt-fbx / tyt-injection. See
[reference/consumer-census.md](reference/consumer-census.md) for the per-file
sites. `voxcore` is untouched. No external wire changes: the fbx JSON is pinned
by the kept DTO; vmax/voxj/goxl/qb pool colors cross their boundaries as raw
`[u8; 4]` / `[f64; N]` / hex strings, not palette types; Oklab and in-gamut sRGB
are numerically identical.

## Commit strategy

The alias flip is ATOMIC: the moment `TySrgba` becomes `palette::Srgba`, every
consumer's `.r` and `.to_f64()` breaks at once, so there is no green intermediate
between "before the flip" and "all consumers fixed." That is one green state
boundary, hence **one clean commit, prepared across sessions** (recommended):
work proceeds crate-by-crate in the working tree, staged not committed, and the
single Conventional Commit lands only when `cargo check --workspace` + clippy +
tests are green. `cargo check -p <crate>` verifies sub-parts along the way. The
"WIP commits then squash" alternative is strictly worse here - the pre-commit
hook runs clippy, so non-green checkpoints would need `--no-verify` - so it is
not recommended.

## Not in scope

- `voxcore`'s `VoxValuePool` model and serde (raw arrays, named variants).
- A palette-REDUCTION / quantization library: `palette` gives color types and
  conversions, NOT median-cut / octree / k-means. `reduce_palette`'s clustering
  stays voxsmith's own; only the color-SPACE placement moves onto palette's
  Oklab/Lab conversions. Evaluating a third-party quantizer is a separate track.
- The vxl `srgb_hex` and voxj `encode_hex` hand-rolled hex from `[u8; 4]` (they
  never used tyt `to_hex`; leave them or fold them into `TyHexColor` later).
