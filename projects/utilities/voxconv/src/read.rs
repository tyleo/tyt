use crate::{Dependencies, ReadFormat, Result, VoxDocumentFile, internal};
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
    match format {
        #[cfg(feature = "goxl")]
        ReadFormat::Goxl => internal::read_goxl(dependencies.goxl(), files),
        #[cfg(feature = "mvox")]
        ReadFormat::MVox => internal::read_mvox(dependencies, files),
        #[cfg(feature = "qbcl")]
        ReadFormat::Qb => internal::read_qb(files),
        #[cfg(feature = "qbcl")]
        ReadFormat::Qbt => internal::read_qbt(dependencies.qbcl(), files),
        #[cfg(feature = "qbcl")]
        ReadFormat::Qbcl => internal::read_qbcl(dependencies.qbcl(), files),
        #[cfg(feature = "vmax")]
        ReadFormat::VMax => internal::read_vmax(dependencies.vmax(), files),
        #[cfg(feature = "voxj")]
        ReadFormat::Voxj => internal::read_voxj(dependencies.voxj(), files),
    }
}
