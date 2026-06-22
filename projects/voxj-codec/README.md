# voxj-codec

Encodes and decodes Voxel Json `.voxj` / `.voxjz` documents.

`encode_file` / `encode_file_smallest` / `decode_file` convert a whole document between the codec form (`voxj::VoxjCodecFile`, objects holding raw positions/samples) and the serde form (`voxj::VoxjSerdeFile`, objects holding encoded blocks): `encode_file` applies fixed block encodings, `encode_file_smallest` tries every encoding pairing and keeps the smallest deflated, and `decode_file` is the inverse. `from_voxj_bytes` / `to_voxj_bytes` parse and serialize a `voxj::VoxjSerdeFile` to and from the uncompressed `.voxj` JSON form; `from_voxjz_bytes` / `to_voxjz_bytes` handle the zip-packaged `.voxjz` form, and `from_voxj_or_voxjz_bytes` accepts either, detecting the form by its leading bytes.

The per-object building blocks underneath: `encode_object` / `encode_object_smallest` / `decode_object` convert a single object between raw positions/samples and the format's `raw-json` / `bitmap-base64` / `hilbert_index-delta-varint-base64` position encodings and `raw-json` / `rle-json` / `packed-base64` sample encodings, and `palette_cell_counts` computes the cell count of each palette an object's `palette_refs` name (the widths `packed-base64` needs). See the `voxj` crate for the format specification.
