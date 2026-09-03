# vmax-codec

Loads and saves Voxel Max `.vmax` packages. The `vmax` crate defines the data
types. This crate turns a package on disk into a `vmax::VMaxFile` and back
without loss.

A `.vmax` package is a directory of files, but the codec never touches the
filesystem itself. You give `from_vmax_package` a way to list and read the
package's files and `to_vmax_package` a way to write them, so the same code works
against a folder, a zip, or bytes already in memory. Reading parses every file
the package can hold into typed fields. An unknown filename or an unmodeled key
is an error, which keeps a round trip lossless.

The codec is a dumb load/save: a `VMaxFile` is the parsed package in its
on-disk shape, and `from_vmax_package` then `to_vmax_package` reproduces it. Voxel
geometry stays as the per-chunk snapshot edit log and palette colors stay packed
or in their PNG. The `vmax` crate's `snapshots` and `palette` modules decode
each on demand.

The QuickLook thumbnails decode to `vmax::VMaxImage` pixel grids, split by role
into `thumbnail_png` for the package preview, `contents_vmax_pngs` for each
object, and `group_pngs` for each group. The `*.vmaxhb`, `*.vmaxhvsb`, and
`*.vmaxhvsc` history files decode into typed sessions and snapshots. The
undocumented per-command undo and redo payloads inside those files stay
`vmax::VMaxValue` trees and round-trip uninterpreted.

## Dependencies

Each load and save function transcodes its payloads through traits:

1. `DecodeVMaxPlist` and `EncodeVMaxPlist` for the binary plists
2. `DecompressLzfse` and `CompressLzfse` for their LZFSE framing
3. `DecodePng` and `EncodePng` for the PNGs
4. `DecodeVMaxSceneJson` and `EncodeVMaxSceneJson` for `scene.json`

Each function binds on what it uses and takes `dependencies` first.
Implement the traits, or use `DependenciesImpl` behind the default `impl`
feature: plists over `plist`, LZFSE over `lzfse`, PNGs over `png`, and JSON
over `serde_json` with exact float parsing. The feature adds those crates
plus `serde` for the derives on `vmax`'s types. A lean build depends on
`vmax` alone.

## Example

Read a `.vmax` directory and write it back out. Only the closures know about the
filesystem, so the same flow works against a zip or an in-memory package.

```rust
use std::{fs, io, path::Path};
use vmax::snapshots::decode_vmax_snapshots;
use vmax_codec::{DependenciesImpl, from_vmax_package, to_vmax_package};

fn round_trip(src: &Path, dst: &Path) -> io::Result<()> {
    let file = from_vmax_package(
        &DependenciesImpl,
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

    to_vmax_package(&DependenciesImpl, &file, |name, bytes| {
        let path = dst.join(name);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, bytes)
    })
}
```

Per-file helpers load and save one file at a time. See the `vmax` crate for
the data types.
