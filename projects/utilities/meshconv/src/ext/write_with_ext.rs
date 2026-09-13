use crate::{
    Dependencies, InstalledFormat, MeshDocumentFile, Result, WriteFormat, WriteFormatVisitor,
    ext::MeshconvMeshMain,
};

/// Encodes a state as a document's files. A box holding the format's ext
/// writes the loaded file back exactly. Any other box writes a file
/// synthesized from the document. The state is consumed because the
/// format's writer needs it in the format's ext type.
pub fn write_with_ext<D: Dependencies>(
    dependencies: &D,
    format: &WriteFormat,
    main: MeshconvMeshMain,
) -> Result<Vec<MeshDocumentFile>> {
    format.with(WriteWithExt { dependencies, main })
}

/// The typed write of one format.
struct WriteWithExt<'a, D> {
    dependencies: &'a D,
    main: MeshconvMeshMain,
}

impl<D: Dependencies> WriteFormatVisitor for WriteWithExt<'_, D> {
    type Output = Result<Vec<MeshDocumentFile>>;

    fn visit<F: InstalledFormat>(self, options: &F::WriteOptions) -> Self::Output {
        F::write_with_ext(self.dependencies, options, self.main)
    }
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, ReadFormat, WriteFormat,
        ext::{MeshconvExt, read_with_ext, write_with_ext},
        test_main, write,
    };
    use gltf_meshdoc::GltfExt;
    use meshdoc::MeshExt;

    /// An ext standing in for another format's.
    #[derive(Clone, Debug)]
    struct Foreign;

    impl MeshExt for Foreign {}

    #[test]
    fn a_format_writes_its_own_ext_back() {
        let format = WriteFormat::from(ReadFormat::Gltf);

        let files = write(&DependenciesImpl, &format, test_main(())).unwrap();

        let loaded = read_with_ext(&DependenciesImpl, ReadFormat::Gltf, &files).unwrap();

        assert!(loaded.ext().is::<GltfExt>());

        let again = write_with_ext(&DependenciesImpl, &format, loaded).unwrap();

        assert_eq!(again, files);
    }

    #[test]
    fn a_foreign_ext_writes_a_synthesized_file() {
        let main = test_main(Box::new(Foreign) as Box<dyn MeshconvExt>);

        let files = write_with_ext(
            &DependenciesImpl,
            &WriteFormat::from(ReadFormat::Gltf),
            main,
        )
        .unwrap();

        let loaded = read_with_ext(&DependenciesImpl, ReadFormat::Gltf, &files).unwrap();

        assert_eq!(loaded.object_count(), 1);
    }
}
