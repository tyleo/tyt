use crate::{Dependencies, ReadFormat, Result, VoxDocumentFile, read};
#[cfg(feature = "voxj")]
use crate::{check_voxj, single_file_bytes};
use voxcore::check::VoxCheck;

/// Checks a document's files: first whether they decode as `format` into a
/// voxcore state, then every spec check the format defines over its
/// encoding. Files that fail to decode fail the `decode` check and run
/// nothing further.
pub fn check_document_files<D: Dependencies>(
    dependencies: &D,
    format: ReadFormat,
    files: &[VoxDocumentFile],
) -> Result<Vec<VoxCheck>> {
    if let Err(error) = read::<D, ()>(dependencies, format, files) {
        return Ok(vec![VoxCheck::failed("decode", vec![error.to_string()])]);
    }

    // The format's spec checks. Only Voxel Json defines any today.
    let format_checks = match format {
        #[cfg(feature = "goxl")]
        ReadFormat::Goxl => Vec::new(),
        #[cfg(feature = "mvox")]
        ReadFormat::MVox => Vec::new(),
        #[cfg(feature = "qbcl")]
        ReadFormat::Qb | ReadFormat::Qbt | ReadFormat::Qbcl => Vec::new(),
        #[cfg(feature = "vmax")]
        ReadFormat::VMax => Vec::new(),
        #[cfg(feature = "voxj")]
        ReadFormat::Voxj => check_voxj(dependencies.voxj(), single_file_bytes(files)?)?,
    };

    let mut checks = vec![VoxCheck::passed("decode")];

    checks.extend(format_checks);

    Ok(checks)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, ReadFormat, VoxDocumentFile, WriteFormat, check_document_files,
        test_state, write,
    };
    use voxcore::check::VoxCheckStatus;

    /// A written Voxel Json document decodes and passes every spec check.
    #[test]
    fn a_written_voxj_document_checks_clean() {
        let files = write(
            &DependenciesImpl,
            &WriteFormat::from(ReadFormat::Voxj),
            test_state(()),
        )
        .unwrap();

        let checks = check_document_files(&DependenciesImpl, ReadFormat::Voxj, &files).unwrap();

        assert_eq!(checks[0].name, "decode");

        assert!(checks.len() > 1);

        assert!(checks.iter().all(|check| {
            matches!(
                check.status,
                VoxCheckStatus::Passed | VoxCheckStatus::Unverifiable
            )
        }));
    }

    /// Bytes no format decodes fail the decode check and nothing else runs.
    #[test]
    fn undecodable_files_fail_decode_only() {
        let files = [VoxDocumentFile::single(b"not a document".to_vec())];

        for format in [ReadFormat::Voxj, ReadFormat::MVox] {
            let checks = check_document_files(&DependenciesImpl, format, &files).unwrap();

            assert_eq!(checks.len(), 1);

            assert!(checks[0].status.is_failed(), "{format:?}");
        }
    }
}
