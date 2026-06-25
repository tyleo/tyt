# voxj-codec

Encodes and decodes Voxel Json `.voxj` / `.voxjz` documents.

`encode_voxj_file` / `encode_voxj_file_smallest` / `decode_voxj_file` convert a whole document between the codec form (`voxj::VoxjCodecFile`, objects holding raw positions/samples) and the serde form (`voxj::VoxjSerdeFile`, objects holding encoded blocks): `encode_voxj_file` applies fixed block encodings, `encode_voxj_file_smallest` tries every encoding pairing and keeps the smallest deflated, and `decode_voxj_file` is the inverse. `from_voxj_file_bytes` / `to_voxj_file_bytes` parse and serialize a `voxj::VoxjSerdeFile` to and from the uncompressed `.voxj` JSON form; `from_voxjz_file_bytes` / `to_voxjz_file_bytes` handle the zip-packaged `.voxjz` form, and `from_voxj_or_voxjz_file_bytes` accepts either, detecting the form by its leading bytes.

The per-object building blocks underneath: `encode_voxj_object` / `encode_voxj_object_smallest` / `decode_voxj_object` convert a single object between raw position and sample encodings, and `voxj_palette_cell_counts` computes the cell count of each palette an object's `palette_refs` name (the widths `packed-base64` needs). See the `voxj` crate for the format specification.
