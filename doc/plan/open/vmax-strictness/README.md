# Vmax strictness

Status: **open.** PR #1, the Voxel Max conformance series, merged on
2026-10-11 as `03afe6ad` through `56a2d729`, with review fixes in `8071a505`.
It left values that read or write silently when they do not fit, which the
[Errors](../../../../CLAUDE.md#errors) rule forbids. This plan makes each one
an error. Each step lands as its own commit with a test.

## 1. Model the contents file (done)

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

`vmax-voxcore/src/from_vmax_file.rs`. Error, naming the object, on:

1. an inverted `vp`, which `edit_grid` reads as an empty grid with `.max(0)`
2. an `eo` outside 5 to 9, which `work_area` reads as 8 with
   `.filter(5..=9).unwrap_or(8)`
3. a `vp` outside its cube, or a corner the `as i32` casts truncate

An absent `eo` still means 8.

Voxel Max keeps voxels outside `vp` and hides them; shrinking the work area in
the app makes them. The reader drops them (`build_object`'s `retain`), so keep
them instead: the edit grid grows to hold them, and the ext records the `vp`
only when it hides voxels. The writer writes a recorded `vp` back, and errors
when it no longer fits the edit grid.

## 3. Expect the workspace fit

`vmax-voxcore/src/internal/place_object_in_workspace.rs`. The writer now picks
a cube holding the grid, yet `centered` still clamps with `.max(0)`. `expect`
the fit instead. Name the 256 in `previous`: it is the width `place_object`
centers a canvas in.

## 4. Check snapshot data lengths

`vmax/src/snapshots/decode_vmax_snapshots.rs`. A `ds` longer than its chunk's
32,768 slots decodes into the next chunk, and an odd length drops its last
byte. `8071a505` fixed both in `decode_vmax_legacy_chunks`. Mirror it here.
This predates PR #1.

## Waiting

To take up after step 4.

1. `VMaxOpaqueFile` writes into `.voxj` as a JSON number array, about 4x its
   bytes. Base64 would shrink it.
2. Removing an external mesh's node leaves its `.scndata` and `.vxtex` in
   `other_files` and its entry in `trsc`.
3. The ext write path errors on a `.voxj` palette edited out of slot layout,
   where the bare path lays it out. This predates PR #1.
