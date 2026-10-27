# mvox

A zero-dependency Rust data model for [MagicaVoxel](https://ephtracy.github.io/) `.vox` files. A scene decodes into typed structs, and files round-trip unchanged. The types live here; reading and writing bytes lives in `mvox-codec`.

The format is documented by ephtracy at
[MagicaVoxel-file-format-vox.txt](https://github.com/ephtracy/voxel-model/blob/master/MagicaVoxel-file-format-vox.txt)
and
[MagicaVoxel-file-format-vox-extension.txt](https://github.com/ephtracy/voxel-model/blob/master/MagicaVoxel-file-format-vox-extension.txt).
