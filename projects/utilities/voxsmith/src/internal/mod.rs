// Palette-color sampling helpers, shared by the formats that re-sample colors
// per voxel (Goxel, MagicaVoxel, Qubicle, Voxel Max), gated by the `_color`
// feature those codecs enable.
#[cfg(feature = "_color")]
mod cell_color;
#[cfg(feature = "_color")]
mod resolve_cell_color;
#[cfg(feature = "_color")]
mod resolve_cell_color_or_transparent;

// Tightening a build-volume object to its live extent is wanted only by the
// Voxel Max writer.
#[cfg(feature = "vmax")]
mod grid;

// Generic mesh-voxelization internal helpers (mesh types and the rasterizer),
// gated by the `_mesh` marker any mesh-format feature enables.
#[cfg(feature = "_mesh")]
mod mesh;

// glTF-output internal helpers (the `VoxObject`-to-glTF document builders),
// gated by the `gltf` writer feature.
#[cfg(feature = "gltf")]
mod gltf;

// Per-codec internal modules.
#[cfg(feature = "vmax")]
mod vmax;

#[cfg(feature = "_color")]
pub(crate) use cell_color::*;
#[cfg(feature = "_color")]
pub(crate) use resolve_cell_color::*;
#[cfg(feature = "_color")]
pub(crate) use resolve_cell_color_or_transparent::*;

#[cfg(feature = "vmax")]
pub(crate) use grid::*;

#[cfg(feature = "_mesh")]
pub(crate) use mesh::*;

#[cfg(feature = "gltf")]
pub(crate) use gltf::*;

#[cfg(feature = "vmax")]
pub(crate) use vmax::*;
