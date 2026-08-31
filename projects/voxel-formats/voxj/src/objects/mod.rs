//! Decodes and encodes the voxel-position and voxel-sample blocks of a
//! [`VoxjObject`](crate::VoxjObject), gated behind the `objects` feature.

mod internal;
pub(crate) use internal::*;

mod decode_voxj_object;
mod encode_voxj_object;
#[cfg(feature = "optimize")]
mod encode_voxj_object_optimized;
mod error;
mod hilbert_bits;
mod max_hilbert_bits;
mod position_encoding;
mod result;
mod sample_encoding;
mod voxj_decoded_object;
mod voxj_palette_material_counts;

pub use decode_voxj_object::*;
pub use encode_voxj_object::*;
#[cfg(feature = "optimize")]
pub use encode_voxj_object_optimized::*;
pub use error::*;
pub use hilbert_bits::*;
pub use max_hilbert_bits::*;
pub use position_encoding::*;
pub use result::*;
pub use sample_encoding::*;
pub use voxj_decoded_object::*;
pub use voxj_palette_material_counts::*;
