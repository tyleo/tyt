// Palette-color sampling helpers, shared by the formats that re-sample colors
// per voxel (Goxel, MagicaVoxel, Qubicle), gated by the `_color` feature those
// codecs enable.
#[cfg(feature = "_color")]
mod cell_color;
#[cfg(feature = "_color")]
mod resolve_cell_color;
#[cfg(feature = "_color")]
mod resolve_cell_color_or_transparent;

// Generic mesh-voxelization internal helpers (mesh types and the rasterizer),
// gated by the `_mesh` marker any mesh-format feature enables.
#[cfg(feature = "_mesh")]
mod mesh;

// glTF-output internal helpers (the `VoxObject`-to-glTF document builders),
// gated by the `gltf` writer feature.
#[cfg(feature = "gltf")]
mod gltf;

// Shared by the inspection reports.
#[cfg(feature = "report")]
mod property_names;

#[cfg(feature = "_color")]
pub(crate) use cell_color::*;
#[cfg(feature = "_color")]
pub(crate) use resolve_cell_color::*;
#[cfg(feature = "_color")]
pub(crate) use resolve_cell_color_or_transparent::*;

#[cfg(feature = "_mesh")]
pub(crate) use mesh::*;

#[cfg(feature = "gltf")]
pub(crate) use gltf::*;

#[cfg(feature = "report")]
pub(crate) use property_names::*;
