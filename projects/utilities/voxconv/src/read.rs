use crate::{
    Dependencies, InstalledFormat, ReadFormat, ReadFormatVisitor, Result, VoxDocumentFile,
};
use voxcore::VoxMain;

/// Decodes a document's files into a bare [`VoxMain`]. The format's ext
/// drops. The `ext` feature's `ext::read_with_ext` keeps it. A single-file
/// format takes exactly one file. A package takes every file
/// [`read_document_files`](crate::read_document_files) lists.
pub fn read<D: Dependencies>(
    dependencies: &D,
    format: ReadFormat,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<()>> {
    format.with(Read {
        dependencies,
        files,
    })
}

/// The bare read of one format.
struct Read<'a, D> {
    dependencies: &'a D,
    files: &'a [VoxDocumentFile],
}

impl<D: Dependencies> ReadFormatVisitor for Read<'_, D> {
    type Output = Result<VoxMain<()>>;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::read(self.dependencies, self.files)
    }
}
