use crate::{
    Dependencies, InstalledFormat, MeshDocumentFile, ReadFormat, ReadFormatVisitor, Result, read,
};
use meshdoc::check::MeshCheck;

/// Checks that a document's files decode as `format`, then runs every spec
/// check the format defines. Files that fail to decode fail the `decode`
/// check and run nothing further.
pub fn check_document_files<D: Dependencies>(
    dependencies: &D,
    format: ReadFormat,
    files: &[MeshDocumentFile],
) -> Result<Vec<MeshCheck>> {
    if let Err(error) = read(dependencies, format, files) {
        return Ok(vec![MeshCheck::failed("decode", vec![error.to_string()])]);
    }

    let mut checks = vec![MeshCheck::passed("decode")];

    checks.extend(format.with(Check {
        dependencies,
        files,
    })?);

    Ok(checks)
}

/// The spec checks of one format.
struct Check<'a, D> {
    dependencies: &'a D,
    files: &'a [MeshDocumentFile],
}

impl<D: Dependencies> ReadFormatVisitor for Check<'_, D> {
    type Output = Result<Vec<MeshCheck>>;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::check(self.dependencies, self.files)
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, MeshDocumentFile, ReadFormat, WriteFormat, check_document_files,
        test_main, write,
    };
    use meshdoc::check::MeshCheckStatus;

    #[test]
    fn a_written_document_checks_clean() {
        let files = write(
            &DependenciesImpl,
            &WriteFormat::from(ReadFormat::Gltf),
            test_main(()),
        )
        .unwrap();

        let checks = check_document_files(&DependenciesImpl, ReadFormat::Gltf, &files).unwrap();

        assert_eq!(checks[0].name, "decode");

        assert_eq!(checks[0].status, MeshCheckStatus::Passed);
    }

    #[test]
    fn undecodable_files_fail_decode_only() {
        let files = [MeshDocumentFile::primary(b"not a document".to_vec())];

        let checks = check_document_files(&DependenciesImpl, ReadFormat::Gltf, &files).unwrap();

        assert_eq!(checks.len(), 1);

        assert!(checks[0].status.is_failed());
    }
}
