# `vxl vox-doc to voxj`

*Part of [`vxl vox-doc to`](README.md) in the [Vxl Command-Line Reference](../../../README.md).*

```
vxl vox-doc to voxj <input> [output] [options]
```

`vox-doc to voxj` writes a voxel-json document and is the canonical place
encodings and containers are chosen, so it is also how a document is re-encoded,
packed, and unpacked. It owns the encoding choice through `--encoding-preset`,
`--position-encoding`, and `--sample-encoding`, and the output container through
`--format json|zip|pretty`. Those options map onto the spec's
[Voxel Encoding](../../../../../../../projects/voxel/formats/voxj/docs/voxel-json-file-format.md#voxel-encoding)
and
[Choosing an Encoding](../../../../../../../projects/voxel/formats/voxj/docs/voxel-json-file-format.md#choosing-an-encoding).

## Re-encoding, packing, and unpacking

These are not separate commands. The `vox-doc to voxj` command already chooses
encodings and containers, so it covers all three:

1. Re-encode or optimize:
   `vxl vox-doc to voxj in.voxj out.voxj --encoding-preset size` rebuilds every
   object with the smallest encoding pairing. Re-encoding positions reorders
   voxels, and `vox-doc to voxj` regenerates the sample channels to match, which
   is the invariant from
   [Voxel Order](../../../../../../../projects/voxel/formats/voxj/docs/voxel-json-file-format.md#voxel-order).
   Pin one block with `--position-encoding` or `--sample-encoding` to search
   only the other.
2. Pack to the shipping form:
   `vxl vox-doc to voxj in.voxj out.voxjz --format zip`.
3. Unpack to plain JSON: `vxl vox-doc to voxj in.voxjz out.voxj`, optionally
   with `--format pretty` for readable output.
