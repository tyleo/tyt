# goxl

A zero-dependency Rust data model for [Goxel](https://goxel.xyz/) `.gox` files. A file decodes into typed structs — image metadata, voxel blocks, layers, materials, cameras, and light. The types live here; reading and writing bytes lives in `goxl-codec`.

A `.gox` file is a `"GOX "` magic and version followed by a flat list of PNG-style chunks. Voxel data lives in shared `16x16x16` `BL16` blocks (each a `64x64` RGBA PNG) that layers reference by index and tile across the scene; the `PREV` preview is a thumbnail PNG. Both decode into pixel/voxel arrays here, so the model holds no opaque PNG bytes.

The format is defined by Goxel's own reader and writer in
[src/formats/gox.c](https://github.com/guillaumechereau/goxel/blob/master/src/formats/gox.c).
