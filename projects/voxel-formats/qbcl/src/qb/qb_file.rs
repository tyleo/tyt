use crate::qb::{QbColorFormat, QbMatrix, QbZAxisOrientation};

/// A Qubicle Binary `.qb` file: header flags and its matrices, in stored order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QbFile {
    /// Format version; the reference exporter writes `257` (`1.1.0.0`).
    pub version: u32,

    /// On-disk color channel order.
    pub color_format: QbColorFormat,

    /// Authoring Z-axis handedness.
    pub z_axis_orientation: QbZAxisOrientation,

    /// Whether voxel data was run-length encoded. Both forms decode to the same
    /// grid; recorded so the choice round-trips.
    pub compressed: bool,

    /// Whether a voxel's visibility byte is a per-face bitmask rather than a
    /// plain solid/empty flag.
    pub visibility_mask_encoded: bool,

    /// Matrices, in stored order.
    pub matrices: Vec<QbMatrix>,
}

impl Default for QbFile {
    fn default() -> Self {
        Self {
            version: 257,
            color_format: QbColorFormat::Rgba,
            z_axis_orientation: QbZAxisOrientation::LeftHanded,
            compressed: false,
            visibility_mask_encoded: false,
            matrices: Vec::new(),
        }
    }
}
