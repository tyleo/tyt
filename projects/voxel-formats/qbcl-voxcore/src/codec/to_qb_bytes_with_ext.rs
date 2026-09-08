use crate::{
    Result,
    ext::{QbVoxMain, to_qb_file_with_ext},
};
use qbcl_codec::qb::to_qb_file_bytes;

/// Writes a [`QbVoxMain`] and its ext to the bytes of a Qubicle Binary `.qb`
/// file, the bytes form of [`to_qb_file_with_ext`] and the inverse of
/// [`from_qb_bytes_with_ext`](crate::codec::from_qb_bytes_with_ext).
pub fn to_qb_bytes_with_ext(state: &QbVoxMain) -> Result<Vec<u8>> {
    let file = to_qb_file_with_ext(state)?;

    Ok(to_qb_file_bytes(&file))
}

#[cfg(test)]
mod tests {
    use crate::{
        codec::{from_qb_bytes_with_ext, to_qb_bytes_with_ext},
        ext::{from_qb_file_with_ext, to_qb_file_with_ext},
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
        let bytes = to_qb_bytes_with_ext(&from_qb_file_with_ext(&file).unwrap()).unwrap();
        let reloaded = from_qb_bytes_with_ext(&bytes).unwrap();
        assert_eq!(to_qb_file_with_ext(&reloaded).unwrap(), file);
    }
}
