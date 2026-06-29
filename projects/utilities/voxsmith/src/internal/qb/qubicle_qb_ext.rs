use crate::QubicleQbMatrix;
use serde::{Deserialize, Serialize};

/// The `qubicle-qb` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Qubicle Binary `.qb` state with no native voxcore home, kept so a file
/// loaded from a `.qb` package can be written back exactly.
///
/// Each matrix's geometry and colors become a native object sharing one
/// palette, placed by a hierarchy node; this holds the header flags and the
/// per-matrix entries, aligned by index with the objects.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct QubicleQbExt {
    /// The format version from the header.
    pub version: u32,

    /// Whether the on-disk color channel order is `BGRA` rather than `RGBA`.
    pub bgra: bool,

    /// Whether the authoring Z axis is right-handed rather than left-handed.
    #[serde(rename = "right-handed")]
    pub right_handed: bool,

    /// Whether voxel data was run-length encoded on disk.
    pub compressed: bool,

    /// Whether a voxel's visibility byte is a per-face bitmask rather than a
    /// plain solid flag.
    #[serde(rename = "visibility-mask-encoded")]
    pub visibility_mask_encoded: bool,

    /// Per-matrix provenance, aligned by index with the objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matrices: Vec<QubicleQbMatrix>,
}
