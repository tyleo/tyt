# vmax

Rust data model for the Voxel Max (`.vmax`) scene format, with optional `serde` support. Types only; it does not read or write files.

A `.vmax` package holds:

- `scene.json`: the scene graph (groups and objects)
- `contents*.vmaxb`: per-object voxel data (chunk snapshots, tool state, brush, camera, sometimes an embedded palette)
- `*.vmaxpsb`: material and palette settings
- `palette*.png`: the color table
- `*.vmaxhb` / `*.vmaxhvsb` / `*.vmaxhvsc`: undo history
- `*.selection.vmaxb`: saved selections
- `QuickLook/`: thumbnails

There is a type for each piece. History is typed sessions and steps; per-command undo/redo payloads vary by command, so they stay as untyped `VMaxValue` and round-trip unchanged. Saved selections are opaque raw bytes. Thumbnails and palette PNGs decode to pixels, not raw PNG bytes.

The `snapshots` module decodes and encodes an object's voxel snapshots, the baked edit log holding its geometry in a `contents*.vmaxb`:

- `decode_vmax_snapshots`: replays the snapshots into model-space voxels
- `encode_vmax_snapshots`: rebuilds the snapshots from voxels, one checkpoint per occupied chunk
- `encode_contents_vmaxb_file_from_voxels`: wraps rebuilt snapshots in a minimal contents file

The `palette` module's `decode_palette_colors` unpacks the color table a `*.vmaxpsb` embeds when a palette ships no sibling `palette*.png`.

Both modules are pure and always compiled.
