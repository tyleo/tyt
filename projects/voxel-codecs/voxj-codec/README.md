# voxj-codec

Reads and writes Voxel Json `.voxj` / `.voxjz` documents. The `voxj` crate defines the data types; this crate loads and saves them.

`from_voxj_file_bytes` / `to_voxj_file_bytes` parse and serialize a `voxj::VoxjFile` to and from the uncompressed `.voxj` JSON form; `from_voxjz_file_bytes` / `to_voxjz_file_bytes` handle the zip-packaged `.voxjz` form, and `from_voxj_or_voxjz_file_bytes` accepts either, detecting the form by its leading bytes. `to_voxj_pretty_file_bytes` writes the pretty-printed JSON.

The load/save path leaves each object's position and sample blocks encoded. Flatten them on demand: `decode_voxj_object` decodes one `voxj::VoxjObject` into a `VoxjDecodedObject` (flat per-voxel positions and per-palette samples), and `encode_voxj_object` / `encode_voxj_object_smallest` go back, the first with fixed block encodings and the second by trying every encoding pairing and keeping the smallest deflated. `voxj_palette_cell_counts` computes the cell count of each palette an object's `palette_refs` name, the widths `packed-base64` needs.

`validate_voxj_file` checks a parsed `voxj::VoxjFile` against the format's document rules (palette and reference resolution, unique in-bounds voxels, acyclic hierarchy, non-degenerate transforms), decoding each object's blocks to run the geometry checks. See the `voxj` crate for the format specification.
