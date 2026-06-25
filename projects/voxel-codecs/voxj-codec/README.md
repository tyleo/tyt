# voxj-codec

Reads, writes, and validates Voxel Json `.voxj` / `.voxjz` documents. The `voxj`
crate defines the data types; this crate is the logic over them.

## Load and save

Load and save leave each object's position and sample blocks encoded.

- `from_voxj_file_bytes` / `to_voxj_file_bytes`: the uncompressed `.voxj` JSON
  form.
- `from_voxjz_file_bytes` / `to_voxjz_file_bytes`: the zip-packaged `.voxjz`
  form.
- `from_voxj_or_voxjz_file_bytes`: either form, detected by its leading bytes.
- `to_voxj_pretty_file_bytes`: pretty-printed `.voxj` JSON.

## Decode and encode blocks

Flatten the encoded blocks on demand, then re-encode.

- `decode_voxj_object`: flatten one object into per-voxel positions and
  per-palette samples.
- `encode_voxj_object`: re-encode with fixed block encodings.
- `encode_voxj_object_smallest`: try every encoding pairing and keep the
  smallest deflated.
- `voxj_palette_cell_counts`: the cell count per referenced palette, the widths
  `packed-base64` needs.

## Validate

`validate_voxj_file` checks a parsed document against the format rules:
reference resolution, unique in-bounds voxels, an acyclic hierarchy, and
non-degenerate transforms, decoding each object to run the geometry checks.

See the `voxj` crate for the format specification.
