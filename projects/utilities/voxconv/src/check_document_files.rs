use crate::{
    Dependencies, InstalledFormat, ReadFormat, ReadFormatVisitor, Result, VoxDocumentFile, read,
};
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
    if let Err(error) = read(dependencies, format, files) {
        return Ok(vec![VoxCheck::failed("decode", vec![error.to_string()])]);
    }

    let mut checks = vec![VoxCheck::passed("decode")];

    checks.extend(format.with(Check {
        dependencies,
        files,
    })?);

    Ok(checks)
}

/// The spec checks of one format.
struct Check<'a, D> {
    dependencies: &'a D,
    files: &'a [VoxDocumentFile],
}

impl<D: Dependencies> ReadFormatVisitor for Check<'_, D> {
    type Output = Result<Vec<VoxCheck>>;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::check(self.dependencies, self.files)
    }
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
