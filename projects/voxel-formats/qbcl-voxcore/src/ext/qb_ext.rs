use crate::ext::QbExtMatrix;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use voxcore::VoxExt;

/// The `qb` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Qubicle Binary `.qb` state with no native voxcore home, kept so a file
/// loaded from a `.qb` package can be written back exactly.
///
/// Each matrix's geometry and colors become a native object sharing one
/// palette, placed by a hierarchy node; this holds the header flags and the
/// per-matrix entries, aligned by index with the objects. The matrices follow
/// the state through the [`VoxExt`](voxcore::VoxExt) hooks.
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

    /// Per-matrix provenance, aligned by index with the objects. An object
    /// retained after the load has `None`. The writer fills it in like a
    /// synthesized matrix.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub matrices: Vec<Option<QbExtMatrix>>,
}

/// The Qubicle Binary ext as a state's ext. The matrices follow the object
/// listing. A retained object takes no entry. The writer fills it in like a
/// synthesized matrix. The visibility bytes do not follow the voxel hooks. A
/// live voxel retained again for a repaint fires the same hook as a new one,
/// so the list cannot tell an insert from a repaint. The writer's count check
/// reports a list out of step.
impl VoxExt for QbExt {
    fn object_did_retain(&mut self, index: usize) {
        self.matrices.insert(index, None);
    }

    fn object_will_release(&mut self, index: usize) {
        self.matrices.remove(index);
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        let matrix = self.matrices.remove(from);
        self.matrices.insert(to, matrix);
    }
}
