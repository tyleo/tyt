use crate::{QubicleQbVoxMain, Result, to_qb_file};
use qbcl_codec::qb::to_qb_file_bytes;

/// Writes a [`QubicleQbVoxMain`] to the bytes of a Qubicle Binary `.qb` file,
/// the bytes form of [`to_qb_file`] and the inverse of
/// [`from_qb_bytes`](crate::codec::from_qb_bytes).
pub fn to_qb_bytes(state: &QubicleQbVoxMain) -> Result<Vec<u8>> {
    let file = to_qb_file(state)?;
    Ok(to_qb_file_bytes(&file))
}

#[cfg(test)]
mod tests {
    use crate::{
        codec::{from_qb_bytes, to_qb_bytes},
        from_qb_file, to_qb_file,
    };
    use qbcl::qb::{QbFile, QbMatrix, QbVoxel};

    /// A file written to bytes reads back to the same file, so the bytes
    /// functions compose the file conversion and the byte codec the right way
    /// round.
    #[test]
    fn round_trips_through_qb_bytes() {
        let file = QbFile {
            matrices: vec![QbMatrix {
                name: "m".to_owned(),
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
            }],
            ..Default::default()
        };
        let bytes = to_qb_bytes(&from_qb_file(&file).unwrap()).unwrap();
        let reloaded = from_qb_bytes(&bytes).unwrap();
        assert_eq!(to_qb_file(&reloaded).unwrap(), file);
    }
}
