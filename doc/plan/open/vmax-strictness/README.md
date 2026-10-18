# Vmax strictness

Status: **open.** PR #1, the Voxel Max conformance series, merged on
2026-10-11 as `03afe6ad` through `56a2d729`, with review fixes in `8071a505`.
It left values that read or write silently when they do not fit, which the
[Errors](../../../../CLAUDE.md#errors) rule forbids. This plan makes each one
an error. Each step lands as its own commit with a test.

## 1. Model the snapshot stats caches

`vmax/src/common/vmax_stats.rs`. `56a2d729` dropped `deny_unknown_fields` from
`VMaxStats` so the `psum`, `spsum`, and `psv` caches current Voxel Max writes
would read, and every other unknown key now reads silently too. Add the three
as optional fields and restore `deny_unknown_fields`. Move the test from
`vmax-codec/src/from_contents_vmaxb_file_bytes.rs` beside the struct.

Open question: does the writer emit the caches? They derive from the voxels,
and Voxel Max opens a file without them.

## 2. Reject a viewport that fits no cube

`vmax-voxcore/src/from_vmax_file.rs`, `contents_workspace_order`. It measures
`tools.vp` with `saturating_sub` and `saturating_add` and falls back with
`unwrap_or(0)` and `unwrap_or(9)`, so a viewport wider than 512 reads as 512
and an inverted one as 32. Error on both with the object and the viewport.

## 3. Reject a bad stored extent order

`vmax-voxcore/src/to_vmax_file.rs`, `object_extent_order`.
`extent_order.filter(5..=9).unwrap_or(8)` writes a `.voxj` value outside 5 to
9 as 256. Error instead. `None` still marks a synthesized object, which starts
at 256.

## 4. Expect the workspace fit

`vmax-voxcore/src/internal/place_object_in_workspace.rs`. Its doc says the
caller checked that the cube holds the grid, then `centered` clamps with
`.max(0)`. `expect` the fit instead. Name the 256 in `previous`: it is the
width `place_object` centers a canvas in.

## 5. Check snapshot data lengths

`vmax/src/snapshots/decode_vmax_snapshots.rs`. A `ds` longer than its chunk's
32,768 slots decodes into the next chunk, and an odd length drops its last
byte. `8071a505` fixed both in `decode_vmax_legacy_chunks`. Mirror it here.
This predates PR #1.

## Waiting for a reason

1. `VMaxOpaqueFile` writes into `.voxj` as a JSON number array, about 4x its
   bytes. Base64 would shrink it.
2. Removing an external mesh's node leaves its `.scndata` and `.vxtex` in
   `other_files` and its entry in `trsc`.
3. `keep_listed_selections` drops an out-of-range `ao` but keeps an in-range
   one that points at a different object after a reorder.
4. The ext write path errors on a `.voxj` palette edited out of slot layout,
   where the bare path lays it out. This predates PR #1.
