#![deny(rustdoc::broken_intra_doc_links)]

//! The boundary of a voxel grid: its faces meshed and the occlusion at their
//! corners.
//!
//! Every algorithm reads its grid through [`SurfaceGrid`]. A
//! [`VoxObject`](voxcore::VoxObject) is one. The mesh and the occlusion stay
//! in grid units on the grid's axes. The caller applies the voxel size and
//! the placement.

// Public API

mod corner_occlusion;
mod inner_corner_occlusion;
mod mesh_grid;
mod mesh_grid_keyed;
mod mesh_occlusion;
mod surface_grid;
mod surface_mesh;
mod surface_method;
mod surface_span;

pub use corner_occlusion::*;
pub use inner_corner_occlusion::*;
pub use mesh_grid::*;
pub use mesh_grid_keyed::*;
pub use mesh_occlusion::*;
pub use surface_grid::*;
pub use surface_mesh::*;
pub use surface_method::*;
pub use surface_span::*;

// Internal API

mod layer_occlusion;

pub(crate) use layer_occlusion::*;

// Test support

#[cfg(test)]
mod test_utilities;
