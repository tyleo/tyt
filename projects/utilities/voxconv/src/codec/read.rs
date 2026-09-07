use crate::{
    ReadFormat, Result, VoxDocumentFile,
    codec::{Dependencies, internal},
};
use voxcore::{VoxMain, ext::VoxExtBlockCodec};

/// Decodes a document's files into a state. The format's ext moves into the
/// slot `T` through its block form. A single-file format takes exactly one
/// file. A package takes every file
/// [`read_document_files`](crate::codec::read_document_files) lists.
pub fn read<D: Dependencies, T: VoxExtBlockCodec>(
    dependencies: &D,
    format: ReadFormat,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<T>> {
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
