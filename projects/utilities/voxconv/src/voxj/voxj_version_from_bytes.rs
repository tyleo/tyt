use crate::{Dependencies, Result};
use voxj_voxcore::codec::voxj_version_from_bytes as raw_voxj_version_from_bytes;

/// The format version of a `.voxj` or `.voxjz` document, read from the bytes
/// because a loaded state drops it. The container form is detected from the
/// leading bytes.
pub fn voxj_version_from_bytes<D: Dependencies>(dependencies: &D, bytes: &[u8]) -> Result<u32> {
    Ok(raw_voxj_version_from_bytes(dependencies.voxj(), bytes)?)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, ReadFormat, WriteFormat, test_state, voxj::voxj_version_from_bytes, write,
    };

    #[test]
    fn a_written_document_reports_its_version() {
        let files = write(
            &DependenciesImpl,
            &WriteFormat::from(ReadFormat::Voxj),
            test_state(()),
        )
        .unwrap();

        assert_eq!(
            voxj_version_from_bytes(&DependenciesImpl, &files[0].bytes).unwrap(),
            1
        );

        assert!(voxj_version_from_bytes(&DependenciesImpl, b"not a document").is_err());
    }
}
