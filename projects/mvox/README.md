# mvox

Core types for the MagicaVoxel (`.vox`) file format: the on-disk data model for a `.vox` scene, covering its models (paired `SIZE` / `XYZI` chunks), the `RGBA` palette, the scene graph (`nTRN` / `nGRP` / `nSHP`), materials (`MATL`), layers (`LAYR`), render settings (`rOBJ`), cameras (`rCAM`), palette notes (`NOTE`), and the palette index map (`IMAP`), plus any chunk this crate does not model, preserved verbatim so a file round-trips without losing it. Most semi-structured `DICT` chunks decode into typed fields, each with a leftover `extra` dictionary that preserves keys this crate does not model; the schema-less `rOBJ` render-settings chunk is kept verbatim as a single attribute dictionary. This crate is the Rust data model; reading and writing `.vox` bytes lives in the companion `mvox-codec` crate.

The format is documented by ephtracy at
[MagicaVoxel-file-format-vox.txt](https://github.com/ephtracy/voxel-model/blob/master/MagicaVoxel-file-format-vox.txt)
and
[MagicaVoxel-file-format-vox-extension.txt](https://github.com/ephtracy/voxel-model/blob/master/MagicaVoxel-file-format-vox-extension.txt).
