use crate::QbExtMatrix;
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};
use std::any::Any;
#[cfg(not(feature = "ext"))]
use voxcore::ext::Error;
#[cfg(feature = "ext")]
use voxcore::ext::encode_entry;
use voxcore::{
    VoxMap,
    ext::{Result, VoxExt},
};

/// The `qb` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Qubicle Binary `.qb` state with no native voxcore home, kept so a file
/// loaded from a `.qb` package can be written back exactly.
///
/// Each matrix's geometry and colors become a native object sharing one
/// palette, placed by a hierarchy node; this holds the header flags and the
/// per-matrix entries, aligned by index with the objects.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct QbExt {
    /// The format version from the header.
    pub version: u32,

    /// Whether the on-disk color channel order is `BGRA` rather than `RGBA`.
    pub bgra: bool,

    /// Whether the authoring Z axis is right-handed rather than left-handed.
    #[cfg_attr(feature = "ext", serde(rename = "right-handed"))]
    pub right_handed: bool,

    /// Whether voxel data was run-length encoded on disk.
    pub compressed: bool,

    /// Whether a voxel's visibility byte is a per-face bitmask rather than a
    /// plain solid flag.
    #[cfg_attr(feature = "ext", serde(rename = "visibility-mask-encoded"))]
    pub visibility_mask_encoded: bool,

    /// Per-matrix provenance, aligned by index with the objects.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub matrices: Vec<QbExtMatrix>,
}

/// The Qubicle Binary ext as a state's ext. Its block is the `qb` entry.
/// Encoding it needs the `ext` feature.
impl VoxExt for QbExt {
    #[cfg(feature = "ext")]
    fn to_vox_ext(&self) -> Result<VoxMap> {
        encode_entry(self)
    }

    #[cfg(not(feature = "ext"))]
    fn to_vox_ext(&self) -> Result<VoxMap> {
        Err(Error::Invalid(
            "the Qubicle Binary ext encodes its block only with the `ext` feature".to_owned(),
        ))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }
}
