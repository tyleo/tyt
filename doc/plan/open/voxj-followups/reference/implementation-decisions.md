# Implementation decisions

_Part of the [Voxj Follow-Up Capabilities Plan](../README.md)._

Code-level decisions made while executing the [checklist](../checklist.md),
recorded as they land. The plan-level decisions and their rationale live in the
[README](../README.md#decisions); this log is for the finer implementation choices
a reviewer of the Rust would want explained, for example how the linear value
threads onto `--color-format`, how `ior` and `transmission` default when a glTF
omits them, and what the vmax verification found at each hop.

No work has landed yet. Add a section under the relevant track as its first chunk
lands.

## Track A: color spaces and encodings in the CLI

Landed the linear color space as a third `--color-format` value.

- **Space and encoding fold into one enum value.** `VoxjColorFormat` and the
  voxsmith `ColorFormat` gain a single `linear-float` / `LinearFloat` variant
  beside `hex` and `float`, rather than a space flag composed with an encoding
  flag. This makes the one illegal pairing, linear plus hex, unrepresentable, so
  checklist item 3's "reject an illegal combination" needs no runtime guard: the
  flag surface never lets it be expressed. The existing
  `color_format_rejects_an_unknown_encoding` test keeps a bare `linear` invalid.
- **The sRGB-to-linear transfer lives in voxsmith, not ty-math.** The decode is a
  private helper (`srgb_to_linear`, with `decode_rgb` / `decode_rgba`) in
  `voxj_value_pool_from_vox_value_pool.rs`, matching the standard sRGB EOTF that
  ty-math's own `srgb_to_linear` uses. It is not added to ty-math because Track
  A's declared blast radius is voxsmith and vxl, and ty-math is published to
  crates.io, where a new public method would diverge from the `0.1.x` on the
  registry. The decode works on the `f64` components directly, never routing
  through the 8-bit path, so no float precision is lost.
- **One write-time hook covers both commands.** The conversion happens where the
  writer emits each `VoxjValuePool`, which `vxl to voxj` and `vxl voxelize` both
  reach through `write_voxj_document`, so threading the choice once serves both.
  A pool voxcore already stores in a linear kind is emitted as float under every
  choice, unchanged; only the two sRGB kinds vary. Alpha carries no gamma, so the
  decode passes the straight alpha through untouched.
- **Two surfaces, one story (item 4).** `--define-attribute` declares a pool's
  color kind when authoring, including `linear-rgb` / `linear-rgba` for HDR
  authoring; `--color-format` picks how the written pools serialize and may
  decode the sRGB kinds to linear at write. The `VoxjColorFormat` doc comment
  states this split so the authoring and writing surfaces do not read as rivals.
- **Coverage.** voxsmith tests pin the transfer against reference linear values,
  prove an sRGB pool decodes to `linear-rgb-float`, keep the straight alpha,
  round-trip sRGB through linear and back within epsilon, preserve an HDR
  component above 1 on an already-linear pool, and leave the sRGB `float` default
  byte-for-byte unchanged. vxl parses `--color-format linear-float`.

## Track B: glTF import fidelity

_Pending._

## Track C: vmax material round-trip verification

_Pending._
