use crate::QbExt;
use qbcl::qb::{QbColorFormat, QbFile, QbZAxisOrientation};

/// The ext of `file`'s header.
pub fn qb_ext_from_file(file: &QbFile) -> QbExt {
    QbExt {
        version: file.version,
        bgra: matches!(file.color_format, QbColorFormat::Bgra),
        right_handed: matches!(file.z_axis_orientation, QbZAxisOrientation::RightHanded),
        compressed: file.compressed,
        visibility_mask_encoded: file.visibility_mask_encoded,
    }
}
