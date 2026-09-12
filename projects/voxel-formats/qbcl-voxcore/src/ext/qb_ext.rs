#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use voxcore::VoxExt;

/// The `qb` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Qubicle Binary `.qb` header, the one part of a `.qb` file with no native
/// voxcore home, kept so a file loaded from a `.qb` package can be written
/// back exactly.
///
/// Each matrix's geometry and colors become a native object sharing one
/// palette, placed by a root hierarchy node carrying the matrix name and
/// position. The writer derives every per-matrix and per-voxel value from
/// the scene. Nothing here follows an entity.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct QbExt {
    /// The format version from the header.
    pub version: u32,

    /// Whether the on-disk color channel order is `BGRA` rather than `RGBA`.
    pub bgra: bool,

    /// Whether the authoring Z axis is right-handed rather than left-handed.
    #[cfg_attr(feature = "serde", serde(rename = "right-handed"))]
    pub right_handed: bool,

    /// Whether voxel data was run-length encoded on disk.
    pub compressed: bool,

    /// Whether a voxel's visibility byte is a per-face bitmask rather than a
    /// plain solid flag.
    #[cfg_attr(feature = "serde", serde(rename = "visibility-mask-encoded"))]
    pub visibility_mask_encoded: bool,
}

/// Nothing per entity, so every hook is a no-op.
impl VoxExt for QbExt {}
