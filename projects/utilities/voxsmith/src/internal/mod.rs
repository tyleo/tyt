// Ext-block serde helpers, shared by every format that stashes a lossless ext
// (all but Voxel Json). The `_ext` feature is auto-enabled by each of them.
#[cfg(feature = "_ext")]
mod ext_for;
#[cfg(feature = "_ext")]
mod from_vox_value;
#[cfg(feature = "_ext")]
mod to_vox_value;

// Palette-color sampling helpers, shared by the formats that re-sample colors
// per voxel (Goxel, MagicaVoxel, Qubicle, Voxel Max), gated by the `_color`
// feature those codecs enable.
#[cfg(feature = "_color")]
mod cell_color;
#[cfg(feature = "_color")]
mod resolve_cell_color;
#[cfg(feature = "_color")]
mod resolve_cell_color_or_transparent;
#[cfg(feature = "_color")]
mod value_pool_color;

// The one sRGB transfer voxsmith applies, at its 8-bit boundaries: the color
// codecs decode and encode palette colors through it, and the mesh side
// decodes texture texels and encodes atlas texels through it.
#[cfg(any(feature = "_color", feature = "_mesh"))]
mod linear_color_from_srgba_u8;
#[cfg(any(feature = "_color", feature = "_mesh"))]
mod srgba_u8_from_linear_color;
#[cfg(any(feature = "_color", feature = "_mesh"))]
mod value_pool_linear_color;

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
#[cfg(feature = "goxl")]
mod goxl;
#[cfg(feature = "mvox")]
mod mvox;
#[cfg(feature = "qbcl")]
mod qbcl;
#[cfg(feature = "vmax")]
mod vmax;
#[cfg(feature = "voxj")]
mod voxj;

#[cfg(feature = "_ext")]
pub(crate) use ext_for::*;
#[cfg(feature = "_ext")]
pub(crate) use from_vox_value::*;
#[cfg(feature = "_ext")]
pub(crate) use to_vox_value::*;

#[cfg(feature = "_color")]
pub(crate) use cell_color::*;
#[cfg(feature = "_color")]
pub(crate) use resolve_cell_color::*;
#[cfg(feature = "_color")]
pub(crate) use resolve_cell_color_or_transparent::*;
#[cfg(feature = "_color")]
pub(crate) use value_pool_color::*;

#[cfg(any(feature = "_color", feature = "_mesh"))]
pub(crate) use linear_color_from_srgba_u8::*;
#[cfg(any(feature = "_color", feature = "_mesh"))]
pub(crate) use srgba_u8_from_linear_color::*;
#[cfg(any(feature = "_color", feature = "_mesh"))]
pub(crate) use value_pool_linear_color::*;

#[cfg(feature = "vmax")]
pub(crate) use grid::*;

#[cfg(feature = "_mesh")]
pub(crate) use mesh::*;

#[cfg(feature = "gltf")]
pub(crate) use gltf::*;

#[cfg(feature = "goxl")]
pub(crate) use goxl::*;
#[cfg(feature = "mvox")]
pub(crate) use mvox::*;
#[cfg(feature = "qbcl")]
pub(crate) use qbcl::*;
#[cfg(feature = "vmax")]
pub(crate) use vmax::*;
#[cfg(feature = "voxj")]
pub(crate) use voxj::*;
