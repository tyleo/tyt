use crate::{
    Dependencies, InstalledFormat, MeshDocumentFile, ReadFormat, ReadFormatVisitor, Result,
};
use meshdoc::MeshMain;

/// Decodes a document's files into a bare [`MeshMain`]. The format's ext
/// drops. The `ext` feature's `ext::read_with_ext` keeps it.
pub fn read<D: Dependencies>(
    dependencies: &D,
    format: ReadFormat,
    files: &[MeshDocumentFile],
) -> Result<MeshMain<()>> {
    format.with(Read {
        dependencies,
        files,
    })
}

/// The bare read of one format.
struct Read<'a, D> {
    dependencies: &'a D,
    files: &'a [MeshDocumentFile],
}

impl<D: Dependencies> ReadFormatVisitor for Read<'_, D> {
    type Output = Result<MeshMain<()>>;

    fn visit<F: InstalledFormat>(self) -> Self::Output {
        F::read(self.dependencies, self.files)
    }
}
