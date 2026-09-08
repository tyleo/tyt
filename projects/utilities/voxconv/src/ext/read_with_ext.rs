use crate::{Dependencies, ReadFormat, Result, VoxDocumentFile, internal};
use voxcore::{VoxMain, ext::VoxExt};

/// Decodes a document's files into a state carrying the format's ext, boxed.
/// A Voxel Json document's `ext` block decodes into a
/// [`CompositeVoxExt`](voxcore::ext::CompositeVoxExt). A single-file format
/// takes exactly one file. A package takes every file
/// [`read_document_files`](crate::read_document_files) lists.
pub fn read_with_ext<D: Dependencies>(
    dependencies: &D,
    format: ReadFormat,
    files: &[VoxDocumentFile],
) -> Result<VoxMain<Box<dyn VoxExt>>> {
    match format {
        #[cfg(feature = "goxl")]
        ReadFormat::Goxl => internal::read_goxl_with_ext(dependencies.goxl(), files),
        #[cfg(feature = "mvox")]
        ReadFormat::MVox => internal::read_mvox_with_ext(dependencies, files),
        #[cfg(feature = "qbcl")]
        ReadFormat::Qb => internal::read_qb_with_ext(files),
        #[cfg(feature = "qbcl")]
        ReadFormat::Qbt => internal::read_qbt_with_ext(dependencies.qbcl(), files),
        #[cfg(feature = "qbcl")]
        ReadFormat::Qbcl => internal::read_qbcl_with_ext(dependencies.qbcl(), files),
        #[cfg(feature = "vmax")]
        ReadFormat::VMax => internal::read_vmax_with_ext(dependencies.vmax(), files),
        #[cfg(feature = "voxj")]
        ReadFormat::Voxj => internal::read_voxj_with_ext(dependencies.voxj(), files),
    }
}
