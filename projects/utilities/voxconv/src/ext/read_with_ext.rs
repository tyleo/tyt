use crate::{
    Dependencies, InstalledFormat, ReadFormat, ReadFormatVisitor, Result, VoxDocumentFile,
    ext::VoxconvVoxMain,
};

/// Decodes a document's files into a state carrying the format's ext, boxed.
/// A Voxel Json document's `ext` block decodes into a `CompositeVoxExt`. A
/// single-file format takes exactly one file. A package takes every file
/// [`read_document_files`](crate::read_document_files) lists.
pub fn read_with_ext<D: Dependencies>(
    dependencies: &D,
    format: ReadFormat,
    files: &[VoxDocumentFile],
) -> Result<VoxconvVoxMain> {
    format.with(ReadWithExt {
        dependencies,
        files,
    })
}

/// The typed read of one format.
struct ReadWithExt<'a, D> {
    dependencies: &'a D,
    files: &'a [VoxDocumentFile],
}

impl<D: Dependencies> ReadFormatVisitor for ReadWithExt<'_, D> {
    type Output = Result<VoxconvVoxMain>;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::read_with_ext(self.dependencies, self.files)
    }
}
