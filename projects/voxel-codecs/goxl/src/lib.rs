#![deny(rustdoc::broken_intra_doc_links)]

//! Core data types for the Goxel (`.gox`) file format.
//!
//! A `.gox` file is a `"GOX "` magic and version followed by a flat list of
//! PNG-style chunks: image metadata (`IMG `), a preview thumbnail (`PREV`),
//! shared voxel blocks (`BL16`), materials (`MATE`), layers (`LAYR`), cameras
//! (`CAMR`), and light settings (`LIGH`). [`GoxlFile`] decodes the whole file
//! into typed fields; the `goxl-codec` crate reads and writes the bytes,
//! including the PNG encoding of blocks and the preview.
//!
//! The format is defined by Goxel's own reader and writer in
//! [`src/formats/gox.c`](https://github.com/guillaumechereau/goxel/blob/master/src/formats/gox.c).

mod goxl_block;
mod goxl_camera;
mod goxl_dict;
mod goxl_file;
mod goxl_image;
mod goxl_layer;
mod goxl_layer_block;
mod goxl_light;
mod goxl_material;
mod goxl_preview;
mod goxl_shape;
mod goxl_unknown_chunk;
mod goxl_voxel;

pub use goxl_block::*;
pub use goxl_camera::*;
pub use goxl_dict::*;
pub use goxl_file::*;
pub use goxl_image::*;
pub use goxl_layer::*;
pub use goxl_layer_block::*;
pub use goxl_light::*;
pub use goxl_material::*;
pub use goxl_preview::*;
pub use goxl_shape::*;
pub use goxl_unknown_chunk::*;
pub use goxl_voxel::*;
