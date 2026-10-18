# Vmax strictness

Status: **closed.** PR #1, the Voxel Max conformance series, merged on
2026-10-11 as `03afe6ad` through `56a2d729`, with review fixes in `8071a505`.
It left values that read or write silently when they do not fit, which the
[Errors](../../../../CLAUDE.md#errors) rule forbids. This plan made each one an
error. Every step landed on main by 2026-10-18 as its own commit with a test.

## 1. Model the contents file

A key Voxel Max needs to open a file is a required field in the model, and an
unknown key is an error.

1. `070d4778`: contents and stats deny unknown keys. Stats model the `psum`,
   `spsum`, and `psv` caches, which the writer omits.
2. `e371f214`: the per-object `cam` is required. The writer frames one when the
   ext has none, `10d6e001` standing it back with the grid. The scene's `ao`,
   then in `ed4e4058` its `ag`, `vl`, and node `s`, read and are never written.
   `17dcb02f` drops `--object-size`.
3. `cbb1acf8`: `tools` is a strict `VMaxTools`. The writer always writes the
   edit grid as `vp` with fresh tool settings, in the smallest cube from 32 to
   512 holding it, and errors on a grid with no volume. The ext no longer
   stores an extent order.

## 2. Check the work area on read

`4324b911`: the reader errors, naming the object, on an inverted `vp`, an `eo`
outside 5 to 9, a `vp` leaving its cube, and a voxel outside the cube. An
absent `eo` still means 8.

Voxel Max keeps voxels outside `vp` and hides them. Shrinking the work area in
the app makes them. The reader keeps them: the edit grid grows to hold them,
and the ext records the `vp` only when it hides voxels. The writer writes a
recorded `vp` back, and errors when it no longer fits the edit grid.

`26421de1` reads the work-area caches and `bbr` list current Voxel Max writes.

## 3. Expect the workspace fit

`efaf14f8`: `place_object_in_workspace` expects the editor cube to hold the
grid it centers, in place of a `.max(0)` clamp.

## 4. Check snapshot data lengths

`3c8fecf1`: a `ds` longer than its chunk's 32,768 slots, or of odd length,
errors, as `8071a505` made `decode_vmax_legacy_chunks` do.

## Follow-ups

1. `754c6df4`: the ext holds each package file the scene does not model as
   base64 text. The `vmax` crate owns the injected `EncodeBase64` and
   `DecodeBase64` dependencies.
2. `05813a7b`: releasing an external mesh's last node drops its `.scndata`,
   its `trsc` entry, and the textures no other mesh uses. The reader skips the
   orphan files Voxel Max keeps for its undo history.
3. `4658c65e`: the ext keeps only what the material pools cannot hold, which
   is a palette's name, its slots stored without dispersion, and its
   transmission colors. The slider map uses Voxel Max's f32 ends, and an
   off-span coefficient errors on read.
4. `731564a3`: a palette edit sets a value in place when every material
   drawing it writes the same result and no other cell draws it. Materials
   sharing a Voxel Max slot then still share it, so the edited palette writes
   back. An edit that splits a slot errors in the writer, naming the palette
   and the material.
