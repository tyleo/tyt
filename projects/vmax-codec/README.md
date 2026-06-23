# vmax-codec

Reads and writes Voxel Max `.vmax` packages and their internal files.

`from_vmax` / `to_vmax` convert a whole `.vmax` directory to and from a `vmax::VMaxSerdeFile`, given a closure that resolves/writes files by name (so the crate needs no filesystem of its own). Per-file helpers (`from_vmaxb_bytes`, `from_vmaxpsb_bytes`, `from_scene_json_bytes`, `from_palette_png_bytes`, and their `to_*` inverses) handle the LZFSE / binary-plist / PNG / JSON framing of each file kind.

`decode_file` / `encode_file` convert a whole package between the serde form (`vmax::VMaxSerdeFile`, raw parsed payloads) and the codec form (`vmax::VMaxCodecFile`, decoded `vmax::VMaxCodecVoxel` geometry and `vmax::VMaxCodecMaterial` palettes), losslessly: the scene graph and `palette*.png` color tables carry over unchanged, and the per-object / per-palette payloads round-trip every field (the snapshot edit-log collapses to its final voxels, and each material's `mi` slot token is reconstructed from its slot position). The per-file building blocks underneath are `decode_contents` / `encode_contents` (voxel snapshots) and `decode_palette_settings` / `encode_palette_settings`, plus the lower-level `decode_snapshots` / `encode_snapshots`. See the `vmax` crate for the data types.
