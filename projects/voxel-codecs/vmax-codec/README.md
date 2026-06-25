# vmax-codec

Reads and writes Voxel Max `.vmax` packages. The `vmax` crate defines the data
types; this crate turns a package on disk into those types and back.

A `.vmax` package is a directory of files, but the codec never touches the
filesystem itself. You give `from_vmax_file` a way to list and read the package's
files and `to_vmax_file` a way to write them, so the same code works against a
folder, a zip, or bytes already in memory. Reading parses every file the package
can hold into a `vmax::VMaxSerdeFile`, and reports an unknown filename instead of
dropping it, so nothing is lost on a round-trip.

To work with the model itself, `decode_vmax_file` turns that serde form into a
`vmax::VMaxCodecFile` of real voxel geometry and materials, and `encode_vmax_file`
turns it back. Both directions are lossless: decoded payloads round-trip every
field, and the streams the format keeps opaque, like undo history and thumbnails,
are preserved byte for byte.

Per-file helpers cover the cases where you need a single file rather than a whole
package. See the `vmax` crate for the data types.

## Example

Read a `.vmax` directory, decode it to the model, and write it back out. Only the
closures know about the filesystem, so the same flow works against a zip or an
in-memory package.

```rust
use std::{fs, io, path::Path};
use vmax_codec::{decode_vmax_file, encode_vmax_file, from_vmax_file, to_vmax_file};

fn round_trip(src: &Path, dst: &Path) -> io::Result<()> {
    let serde = from_vmax_file(
        || list_files(src), // your lister, returning package-relative paths
        |name| match fs::read(src.join(name)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            result => result.map(Some),
        },
    )?;

    // Decode to voxel geometry and materials. Edit `codec` here, then re-encode.
    let codec = decode_vmax_file(&serde)?;
    let serde = encode_vmax_file(&codec);

    to_vmax_file(&serde, |name, bytes| {
        let path = dst.join(name);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, bytes)
    })
}
```
