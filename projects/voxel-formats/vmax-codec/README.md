# vmax-codec

Loads and saves Voxel Max `.vmax` packages. The `vmax` crate defines the data
types; this crate turns a package on disk into a `vmax::VMaxFile` and back,
losslessly.

A `.vmax` package is a directory of files, but the codec never touches the
filesystem itself. You give `from_vmax_package` a way to list and read the
package's files and `to_vmax_package` a way to write them, so the same code works
against a folder, a zip, or bytes already in memory. Reading parses every file
the package can hold into typed fields, and reports an unknown filename (or an
unmodeled key) instead of dropping it, so nothing is lost on a round trip.

The codec is a dumb load/save: a `VMaxFile` is the parsed package in its
on-disk shape, and `from_vmax_package` then `to_vmax_package` reproduces it. Voxel
geometry stays as the per-chunk snapshot edit log and palette colors stay packed
or in their PNG; decode them on demand with the free functions:

- `decode_vmax_snapshots` replays a `contents*.vmaxb`'s snapshots into voxels,
  and `encode_vmax_snapshots` / `encode_contents_vmaxb_file_from_voxels` go back.
- `decode_palette_colors` unpacks a palette settings file's embedded color table.

The QuickLook thumbnails decode to `vmax::VMaxImage` pixel grids, split by role
into `thumbnail_png` (the package preview), `contents_vmax_pngs` (per object),
and `group_pngs` (per group). The history files (`*.vmaxhb` / `*.vmaxhvsb` /
`*.vmaxhvsc`) decode into typed sessions and snapshots; the undocumented
per-command undo/redo payloads inside them are held as `vmax::VMaxValue` trees
so they round-trip without being interpreted.

## Example

Read a `.vmax` directory and write it back out. Only the closures know about the
filesystem, so the same flow works against a zip or an in-memory package.

```rust
use std::{fs, io, path::Path};
use vmax_codec::{decode_vmax_snapshots, from_vmax_package, to_vmax_package};

fn round_trip(src: &Path, dst: &Path) -> io::Result<()> {
    let file = from_vmax_package(
        || list_files(src), // your lister, returning package-relative paths
        |name| match fs::read(src.join(name)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            result => result.map(Some),
        },
    )?;

    // Decode geometry on demand, e.g. the first object's voxels:
    if let Some(contents) = file.contents_files.values().next() {
        let _voxels = decode_vmax_snapshots(&contents.snapshots)?;
    }

    to_vmax_package(&file, |name, bytes| {
        let path = dst.join(name);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, bytes)
    })
}
```

Per-file helpers cover the cases where you need a single file rather than a
whole package. See the `vmax` crate for the data types.
