use crate::{Result, ext::QbVoxMain, write_qb};
use qbcl::qb::QbFile;

/// Writes a [`QbVoxMain`] back to a decoded Qubicle Binary [`QbFile`], the
/// inverse of [`from_qb_file_with_ext`](crate::ext::from_qb_file_with_ext)
/// and the typed form of [`to_qb_file`](crate::to_qb_file). A loaded file
/// writes back exactly through its ext. A state carrying none writes a
/// synthesized file.
///
/// Errors if the ext's matrix entries do not line up with the objects or a
/// visibility list does not match its object.
pub fn to_qb_file_with_ext(state: &QbVoxMain) -> Result<QbFile> {
    write_qb(state, state.ext().as_ref())
}

#[cfg(test)]
mod tests {
    use crate::ext::{QbVoxMain, from_qb_file_with_ext, to_qb_file_with_ext};
    use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};

    /// A file with two matrices: a `[2, 1, 1]` grid with two solid voxels, one
    /// carrying a non-default visibility mask, and a single-voxel grid.
    fn sample_file() -> QbFile {
        QbFile {
            version: 257,
            color_format: QbColorFormat::Bgra,
            z_axis_orientation: QbZAxisOrientation::RightHanded,
            compressed: true,
            visibility_mask_encoded: true,
            matrices: vec![
                QbMatrix {
                    name: "m0".to_owned(),
                    size: [2, 1, 1],
                    position: [1, 2, 3],
                    voxels: vec![
                        QbVoxel::new(10, 20, 30),
                        QbVoxel {
                            r: 1,
                            g: 2,
                            b: 3,
                            visibility: 0x3f,
                        },
                    ],
                },
                QbMatrix {
                    name: "m1".to_owned(),
                    size: [1, 1, 1],
                    position: [-1, -1, -1],
                    voxels: vec![QbVoxel::new(40, 50, 60)],
                },
            ],
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_qb_file_with_ext(&file).unwrap();
        assert_eq!(to_qb_file_with_ext(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbFile::default();
        let state = from_qb_file_with_ext(&file).unwrap();
        assert_eq!(to_qb_file_with_ext(&state).unwrap(), file);
    }

    /// A state carrying no ext writes a synthesized file.
    #[test]
    fn synthesizes_without_an_ext() {
        let file = to_qb_file_with_ext(&QbVoxMain::default()).unwrap();
        assert!(file.matrices.is_empty());
    }
}
