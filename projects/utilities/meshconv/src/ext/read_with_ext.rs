use crate::{
    Dependencies, InstalledFormat, MeshDocumentFile, ReadFormat, ReadFormatVisitor, Result,
    ext::MeshconvMeshMain,
};

/// Decodes a document's files into a state carrying the format's ext, boxed.
pub fn read_with_ext<D: Dependencies>(
    dependencies: &D,
    format: ReadFormat,
    files: &[MeshDocumentFile],
) -> Result<MeshconvMeshMain> {
    format.with(ReadWithExt {
        dependencies,
        files,
    })
}

/// The typed read of one format.
struct ReadWithExt<'a, D> {
    dependencies: &'a D,
    files: &'a [MeshDocumentFile],
}

impl<D: Dependencies> ReadFormatVisitor for ReadWithExt<'_, D> {
    type Output = Result<MeshconvMeshMain>;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::read_with_ext(self.dependencies, self.files)
    }
}
