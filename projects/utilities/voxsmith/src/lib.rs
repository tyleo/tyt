#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! Every operation runs over voxcore's [`VoxMain`](voxcore::VoxMain) and never
//! sees a file format or the filesystem. voxconv decodes a document into a
//! state and encodes one back. `operations` holds a module per vxl command,
//! each behind a feature of the same name. `info`, `validate`,
//! `hierarchy_show`, `palette_list`, and `palette_show` render reports through
//! treegrid. `mesh` writes one object, picked by hierarchy path or index, to
//! glTF. `voxelize` turns a glTF mesh into a state. `utilities` holds what the
//! operations share: palette reduction, dithering, color spaces, the index and
//! vector selectors, the glTF property ranges, and a palette's property names.
//! The mesh writer's PNG and base64 encoders come from the caller through
//! [`EncodePng`](dependencies::mesh::EncodePng) and
//! [`EncodeBase64`](dependencies::mesh::EncodeBase64).
//! [`DependenciesImpl`](dependencies::DependenciesImpl), behind the `impl`
//! feature, binds them over `png` and `base64`.

// Public API

pub mod dependencies;
pub mod operations;
pub mod utilities;

mod error;
mod result;

pub use error::*;
pub use result::*;
