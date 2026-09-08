use crate::{Result, ext::QbclVoxMain, write_qbcl};
use qbcl::qbcl::QbclFile;

/// Writes a [`QbclVoxMain`] back to a decoded Qubicle Construction Library
/// [`QbclFile`], the inverse of
/// [`from_qbcl_file_with_ext`](crate::ext::from_qbcl_file_with_ext) and the
/// typed form of [`to_qbcl_file`](crate::to_qbcl_file). A loaded file writes
/// back exactly through its ext. A state carrying none writes a synthesized
/// file.
///
/// Errors if the ext's node entries do not line up with the hierarchy, the
/// state does not have exactly one root, or a mask list does not match its
/// object.
pub fn to_qbcl_file_with_ext(state: &QbclVoxMain) -> Result<QbclFile> {
    write_qbcl(state, state.ext().as_ref())
}

#[cfg(test)]
mod tests {
    use crate::ext::{from_qbcl_file_with_ext, to_qbcl_file_with_ext};
    use qbcl::qbcl::{
        QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode,
        QbclNodeBody, QbclThumbnail, QbclVoxel,
    };

    /// A matrix node with two solid voxels in a `[2, 1, 1]` grid.
    fn matrix_node() -> QbclNode {
        QbclNode {
            name: "matrix".to_owned(),
            visible: true,
            locked: false,
            body: QbclNodeBody::Matrix(QbclMatrix {
                size: [2, 1, 1],
                position: [1, 2, 3],
                pivot: [0.5, 0.0, 0.0],
                voxels: vec![
                    QbclVoxel::new(10, 20, 30, 0x7e),
                    QbclVoxel::new(1, 2, 3, 0x01),
                ],
            }),
        }
    }

    /// A compound node carrying one baked voxel and one empty model child.
    fn compound_node() -> QbclNode {
        QbclNode {
            name: "compound".to_owned(),
            visible: false,
            locked: true,
            body: QbclNodeBody::Compound(QbclCompound {
                matrix: QbclMatrix {
                    size: [1, 1, 1],
                    position: [-1, -2, -3],
                    pivot: [0.0, 0.0, 0.0],
                    voxels: vec![QbclVoxel::new(40, 50, 60, 0xff)],
                },
                children: vec![QbclNode {
                    name: "leaf".to_owned(),
                    visible: true,
                    locked: false,
                    body: QbclNodeBody::Model(QbclModel::default()),
                }],
            }),
        }
    }

    /// A file exercising a model root grouping a matrix and a compound, with a
    /// thumbnail, metadata, and a guid.
    fn sample_file() -> QbclFile {
        QbclFile {
            program_version: 0x0102_0304,
            file_version: 2,
            thumbnail: QbclThumbnail {
                width: 2,
                height: 1,
                pixels: vec![QbclColor::new(1, 2, 3, 4), QbclColor::new(5, 6, 7, 8)],
            },
            metadata: QbclMetadata {
                title: "Title".to_owned(),
                description: "Desc".to_owned(),
                tags: String::new(),
                author: "Author".to_owned(),
                company: String::new(),
                website: String::new(),
                copyright: "2026".to_owned(),
            },
            guid: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            root: QbclNode {
                name: "root".to_owned(),
                visible: true,
                locked: false,
                body: QbclNodeBody::Model(QbclModel {
                    transform: QbclModel::DEFAULT_TRANSFORM,
                    children: vec![matrix_node(), compound_node()],
                }),
            },
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_qbcl_file_with_ext(&file).unwrap();
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbclFile::default();
        let state = from_qbcl_file_with_ext(&file).unwrap();
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), file);
    }

    /// A file whose root is a matrix rather than the conventional model.
    #[test]
    fn round_trips_a_matrix_root() {
        let file = QbclFile {
            root: matrix_node(),
            ..Default::default()
        };
        let state = from_qbcl_file_with_ext(&file).unwrap();
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), file);
    }
}
