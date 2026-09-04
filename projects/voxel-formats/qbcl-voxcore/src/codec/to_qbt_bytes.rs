use crate::{QbtVoxMain, Result, to_qbt_file};
use qbcl_codec::{CompressZlib, qbt::to_qbt_file_bytes};

/// Writes a [`QbtVoxMain`] to the bytes of a Qubicle Binary Tree
/// `.qbt` file through `dependencies`, the bytes form of [`to_qbt_file`] and
/// the inverse of [`from_qbt_bytes`](crate::codec::from_qbt_bytes).
pub fn to_qbt_bytes<D: CompressZlib>(dependencies: &D, state: &QbtVoxMain) -> Result<Vec<u8>> {
    let file = to_qbt_file(state)?;
    Ok(to_qbt_file_bytes(dependencies, &file))
}

#[cfg(test)]
mod tests {
    use crate::{
        codec::{from_qbt_bytes, to_qbt_bytes},
        from_qbt_file, to_qbt_file,
    };
    use qbcl::qbt::{QbtFile, QbtMatrix, QbtModel, QbtNode, QbtVoxel};
    use qbcl_codec::DependenciesImpl;

    /// A file written to bytes reads back to the same file, so the bytes
    /// functions compose the file conversion and the byte codec the right way
    /// round.
    #[test]
    fn round_trips_through_qbt_bytes() {
        let file = QbtFile {
            root: QbtNode::Model(QbtModel {
                children: vec![QbtNode::Matrix(QbtMatrix {
                    name: "matrix".to_owned(),
                    position: [1, 2, 3],
                    local_scale: [1, 1, 1],
                    pivot: [0.5, 0.0, 0.0],
                    size: [2, 1, 1],
                    voxels: vec![
                        QbtVoxel::new(10, 20, 30, 0x7e),
                        QbtVoxel::new(1, 2, 3, 0x01),
                    ],
                })],
            }),
            ..Default::default()
        };
        let bytes = to_qbt_bytes(&DependenciesImpl, &from_qbt_file(&file).unwrap()).unwrap();
        let reloaded = from_qbt_bytes(&DependenciesImpl, &bytes).unwrap();
        assert_eq!(to_qbt_file(&reloaded).unwrap(), file);
    }
}
