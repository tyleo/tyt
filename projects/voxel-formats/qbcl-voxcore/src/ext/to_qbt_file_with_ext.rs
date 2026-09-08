use crate::{Result, ext::QbtVoxMain, write_qbt};
use qbcl::qbt::QbtFile;

/// Writes a [`QbtVoxMain`] back to a decoded Qubicle Binary Tree
/// [`QbtFile`], the inverse of
/// [`from_qbt_file_with_ext`](crate::ext::from_qbt_file_with_ext) and the
/// typed form of [`to_qbt_file`](crate::to_qbt_file). A loaded file writes
/// back exactly through its ext.
///
/// Errors if the state carries no ext, its node entries do not line up with
/// the hierarchy, the state does not have exactly one root, or a mask list
/// does not match its object.
pub fn to_qbt_file_with_ext(state: &QbtVoxMain) -> Result<QbtFile> {
    write_qbt(state, state.ext().as_ref())
}

#[cfg(test)]
mod tests {
    use crate::ext::{QbtVoxMain, from_qbt_file_with_ext, to_qbt_file_with_ext};
    use qbcl::qbt::{
        QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
    };

    /// A matrix node with two solid voxels in a `[2, 1, 1]` grid.
    fn matrix_node() -> QbtNode {
        QbtNode::Matrix(QbtMatrix {
            name: "matrix".to_owned(),
            position: [1, 2, 3],
            local_scale: [1, 1, 1],
            pivot: [0.5, 0.0, 0.0],
            size: [2, 1, 1],
            voxels: vec![
                QbtVoxel::new(10, 20, 30, 0x7e),
                QbtVoxel::new(1, 2, 3, 0x01),
            ],
        })
    }

    /// A compound node carrying one baked voxel and one empty model child.
    fn compound_node() -> QbtNode {
        QbtNode::Compound(QbtCompound {
            matrix: QbtMatrix {
                name: "compound".to_owned(),
                position: [-1, -2, -3],
                local_scale: [2, 2, 2],
                pivot: [0.0, 0.0, 0.0],
                size: [1, 1, 1],
                voxels: vec![QbtVoxel::new(40, 50, 60, 0xff)],
            },
            children: vec![QbtNode::Model(QbtModel::default())],
        })
    }

    /// A file exercising a model root grouping a matrix, a compound, and an
    /// unknown node, with a color map and a global scale.
    fn sample_file() -> QbtFile {
        QbtFile {
            version: (1, 0),
            global_scale: [1.0, 2.0, 0.5],
            color_map: vec![QbtColor::new(10, 20, 30, 255), QbtColor::new(1, 2, 3, 255)],
            root: QbtNode::Model(QbtModel {
                children: vec![
                    matrix_node(),
                    compound_node(),
                    QbtNode::Unknown(QbtUnknownNode {
                        type_id: 99,
                        data: vec![9, 8, 7],
                    }),
                ],
            }),
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_qbt_file_with_ext(&file).unwrap();
        assert_eq!(to_qbt_file_with_ext(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbtFile::default();
        let state = from_qbt_file_with_ext(&file).unwrap();
        assert_eq!(to_qbt_file_with_ext(&state).unwrap(), file);
    }

    /// A file whose root is a matrix rather than the conventional model.
    #[test]
    fn round_trips_a_matrix_root() {
        let file = QbtFile {
            root: matrix_node(),
            ..Default::default()
        };
        let state = from_qbt_file_with_ext(&file).unwrap();
        assert_eq!(to_qbt_file_with_ext(&state).unwrap(), file);
    }

    #[test]
    fn errors_without_qbt_ext() {
        let state = QbtVoxMain::default();
        assert!(to_qbt_file_with_ext(&state).is_err());
    }
}
