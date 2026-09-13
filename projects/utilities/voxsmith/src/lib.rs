#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! Every operation runs over voxcore's [`VoxMain`](voxcore::VoxMain) and never
//! sees a file format or the filesystem. voxconv decodes a document into a
//! main and encodes one back. `operations` holds a module per vxl command,
//! each behind a feature of the same name. `info`, `validate`,
//! `hierarchy_show`, `palette_list`, and `palette_show` render reports through
//! treegrid. `mesh_old` triangulates one object into a meshdoc document. `to`
//! prunes a main to a chosen set of objects for conversion. `voxelize` turns
//! a meshdoc document into a main. Neither sees a mesh file format; meshconv
//! decodes a mesh document into a `MeshMain` and encodes one back.
//! `utilities` holds what the operations share: palette reduction, dithering,
//! color spaces, the object, index, and vector selectors, the material
//! property ranges, and a palette's property names.
//! The mesh bake's PNG encoder comes from the caller through
//! [`EncodePng`](dependencies::mesh::EncodePng), and the voxelizer's image
//! decoder through [`DecodeImage`](dependencies::voxelize::DecodeImage).
//! [`DependenciesImpl`](dependencies::DependenciesImpl), behind the `impl`
//! feature, binds them over `png` and `zune-jpeg`.

// Public API

pub mod dependencies;
pub mod operations;
pub mod utilities;

mod error;
mod result;

pub use error::*;
pub use result::*;
