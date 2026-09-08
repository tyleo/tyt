use crate::{
    Result,
    ext::{QbclVoxMain, to_qbcl_file_with_ext},
};
use qbcl_codec::{CompressZlib, qbcl::to_qbcl_file_bytes};

/// Writes a [`QbclVoxMain`] and its ext to the bytes of a Qubicle
/// Construction Library `.qbcl` file through `dependencies`, the bytes form
/// of [`to_qbcl_file_with_ext`] and the inverse of
/// [`from_qbcl_bytes_with_ext`](crate::codec::from_qbcl_bytes_with_ext).
pub fn to_qbcl_bytes_with_ext<D: CompressZlib>(
    dependencies: &D,
    state: &QbclVoxMain,
) -> Result<Vec<u8>> {
    let file = to_qbcl_file_with_ext(state)?;

    Ok(to_qbcl_file_bytes(dependencies, &file))
}

#[cfg(test)]
mod tests {
    use crate::{
        codec::{from_qbcl_bytes_with_ext, to_qbcl_bytes_with_ext},
        ext::{from_qbcl_file_with_ext, to_qbcl_file_with_ext},
    };
    use qbcl::qbcl::{QbclFile, QbclMatrix, QbclModel, QbclNode, QbclNodeBody, QbclVoxel};
    use qbcl_codec::DependenciesImpl;

    /// A file written to bytes reads back to the same file, so the bytes
    /// functions compose the file conversion and the byte codec the right way
    /// round.
    #[test]
    fn round_trips_through_qbcl_bytes() {
        let file = QbclFile {
            root: QbclNode {
                name: "root".to_owned(),
                visible: true,
                locked: false,
                body: QbclNodeBody::Model(QbclModel {
                    transform: QbclModel::DEFAULT_TRANSFORM,
                    children: vec![QbclNode {
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
                    }],
                }),
            },
            ..Default::default()
        };
        let state = from_qbcl_file_with_ext(&file).unwrap();
        let bytes = to_qbcl_bytes_with_ext(&DependenciesImpl, &state).unwrap();
        let reloaded = from_qbcl_bytes_with_ext(&DependenciesImpl, &bytes).unwrap();
        assert_eq!(to_qbcl_file_with_ext(&reloaded).unwrap(), file);
    }
}
