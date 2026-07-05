# `vxl to voxj`

*Part of [`vxl to`](README.md) in the [Vxl Command-Line Reference](../../README.md).*

```
vxl to voxj <input> [output] [options]
```

`to voxj` writes a voxel-json document and is the canonical place encodings and
containers are chosen, so it is also how a document is re-encoded, packed, and
unpacked. It owns the encoding choice through `--encoding-preset`,
`--position-encoding`, `--sample-encoding`, and `--color-format`, and the output
container through `--format json|zip|pretty`. `--color-format hex|float` chooses
how sRGB color pools serialize and defaults to `float`; linear color pools always
serialize as float regardless of the choice. Those options map onto the spec's
[Voxel Encoding](../../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#voxel-encoding)
and [Choosing an Encoding](../../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#choosing-an-encoding).

## Re-encoding, packing, and unpacking

These are not separate commands. The `to voxj` command already chooses
encodings and containers, so it covers all three:

1. Re-encode or optimize: `vxl to voxj in.voxj out.voxj --encoding-preset size`
   rebuilds every object with the smallest encoding pairing. Re-encoding
   positions reorders voxels, and `to voxj` regenerates the sample channels to
   match, which is the invariant from
   [Voxel Order](../../../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#voxel-order).
   Pin one block with `--position-encoding` or `--sample-encoding` to search
   only the other.
2. Pack to the shipping form: `vxl to voxj in.voxj out.voxjz --format zip`.
3. Unpack to plain JSON: `vxl to voxj in.voxjz out.voxj`, optionally with
   `--format pretty` for readable output.
