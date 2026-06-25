# goxl

A Rust data model for [Goxel](https://goxel.xyz/) `.gox` files. A scene decodes into typed structs, and files round-trip unchanged. The types live here; reading and writing bytes lives in `goxl-codec`.

The format is defined by Goxel's own reader and writer in
[src/formats/gox.c](https://github.com/guillaumechereau/goxel/blob/master/src/formats/gox.c).

Not yet implemented; this crate is currently an empty placeholder.
