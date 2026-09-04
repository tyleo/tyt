# goxl-codec

Reads and writes [Goxel](https://goxel.xyz/) `.gox` files.

`from_gox_file_bytes` parses bytes into a `goxl::GoxlFile`. `to_gox_file_bytes` writes one back to an equivalent file. Bounds-checked reads reject truncated or malformed input. The shared `BL16` voxel blocks and the `PREV` preview decode from their PNGs into voxel and pixel arrays and re-encode on write. PNG is lossless for the pixels, so a file round-trips through a decode and encode.

`validate_gox_file` checks a file for shape and cross-reference faults. Decoding does not call it.

## Dependencies

The block and preview PNGs go through two traits, `DecodePng` to read and `EncodePng` to write. Both exchange a `GoxlRgbaImage`. Each function binds on what it uses and takes `dependencies` first. Implement the traits, or use `DependenciesImpl` behind the default `impl` feature, which decodes and encodes over `png`. A lean build depends on `goxl` alone.

## Example

```rust
use goxl_codec::{DependenciesImpl, from_gox_file_bytes, to_gox_file_bytes};

fn round_trip(bytes: &[u8]) -> goxl_codec::Result<Vec<u8>> {
    let file = from_gox_file_bytes(&DependenciesImpl, bytes)?;
    Ok(to_gox_file_bytes(&DependenciesImpl, &file))
}
```

See `goxl` for the data types.
