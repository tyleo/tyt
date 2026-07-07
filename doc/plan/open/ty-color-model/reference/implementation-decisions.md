# Implementation decisions

Durable log of non-obvious code-level calls made while executing the
[plan](../README.md). Each entry is a decision a later session must stay
consistent with; read this before picking up work.

## S1: add `TySrgba<T = f32>` (2026-07-07)

- **File / alias layout.** `TySrgba` lives in `ty_srgba.rs`; the aliases are
  `TySrgbaU8` / `TySrgbaF32` / `TySrgbaF64` in `ty_srgba_u8.rs` /
  `ty_srgba_f32.rs` / `ty_srgba_f64.rs`, matching the one-type-per-file rule and
  the existing `Ty*ColorF32/F64` alias-file convention. The `u8` alias sorts
  after `f64` in `lib.rs` because the module list is strictly alphabetical
  (`ty_srgba`, `ty_srgba_color`, `ty_srgba_f32`, `ty_srgba_f64`, `ty_srgba_u8`).

- **`Eq` / `Hash` for `TySrgba<u8>` only.** `PartialEq` is derived generically;
  `Eq` is an empty impl on `TySrgba<u8>` and `Hash` is a hand-written field-order
  impl on `TySrgba<u8>` that mirrors what a derive would emit. `f32` / `f64` get
  `PartialEq` alone. This keeps the dedup key (`MaterialKey`,
  `voxelize_mesh.rs:424`) on the 8-bit storage. clippy is clean: the
  `derived_hash_with_manual_eq` lint fires on derived-`Hash` + manual-`PartialEq`,
  which is the opposite arrangement, so manual `Hash` + derived `PartialEq` is
  fine as long as they agree (they do).

- **Ported surface is exactly the checklist's: `new`, the array conversions (the
  generic `ty_array_conversions!` form: `to_array` / `from_array` / `from_slice`
  / `write_to_slice` + `From<[T; 4]>`, but NOT the reverse `From<Self> for
  [T; 4]`, which the macro emits only for the concrete form), `from_hex` /
  `to_hex` on `TySrgba<u8>`, and the forward `Mul<T>` (color * scalar).**

- **Omitted the scalar-first `Mul` (`scalar * color`).** The old
  `TyRgbaColor` carries a `impl Mul<TyRgbaColor<$t>> for $t` reverse impl; a
  workspace grep (`* Ty(Rgba|Srgba|Linear)`) found zero call sites, so it is dead
  and not ported. Re-add it additively only if a consumer migration turns one up.

- **Deferred `to_vector3` out of S1.** It is not in the checklist's S1 list, and
  its semantics are component-type-dependent: `TySrgbaColor::to_vector3`
  normalizes (u8 -> `TyVector3<f64>` via `to_rgba`) while
  `TyRgbaColor::to_vector3` is a plain drop-alpha. A uniform generic method
  cannot express both, so the call sites (reduce_palette, voxelize_mesh,
  from_qbcl_file) are migrated with the component conversions in S2 / the
  consumer steps, not here.

- **Doc header avoids linking `to_lin_srgba`.** That method lands in S2, so the
  S1 doc describes the decode conceptually ("decode to linear light before
  compositing") rather than with a `[`...`](Self::...)` intra-doc link that would
  not resolve yet. Add the link in S2.
