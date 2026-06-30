// Ext-block serde helpers, shared by every format that stashes a lossless ext
// (all but Voxel Json). The `_ext` feature is auto-enabled by each of them.
#[cfg(feature = "_ext")]
mod ext_for;
#[cfg(feature = "_ext")]
mod from_vox_value;
#[cfg(feature = "_ext")]
mod to_vox_value;

// Palette-color sampling helpers, shared by the formats that re-sample colors
// per voxel (Goxel, MagicaVoxel, Qubicle), gated by the `_color` feature those
// codecs enable. Voxel Max samples colors only in its tests, so they are also
// compiled for a Voxel-Max-only test build.
#[cfg(any(feature = "_color", all(feature = "vmax", test)))]
mod cell_color;
#[cfg(any(feature = "_color", all(feature = "vmax", test)))]
mod object_color_ref;
#[cfg(any(feature = "_color", all(feature = "vmax", test)))]
mod parse_color_hex;

// Tightening a build-volume object to its live extent is wanted only by the
// Voxel Max writer.
#[cfg(feature = "vmax")]
mod grid;

// Mesh-voxelization internal helpers, gated by the `gltf` feature.
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

#[cfg(any(feature = "_color", all(feature = "vmax", test)))]
pub(crate) use cell_color::*;
#[cfg(any(feature = "_color", all(feature = "vmax", test)))]
pub(crate) use object_color_ref::*;
#[cfg(any(feature = "_color", all(feature = "vmax", test)))]
pub(crate) use parse_color_hex::*;

#[cfg(feature = "vmax")]
pub(crate) use grid::*;

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
