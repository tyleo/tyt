use crate::{Dependencies, Format, MeshDocumentFile, Result, ext::MeshconvMeshMain};

/// A format's typed path, which keeps the format's ext across a read and a
/// write. Each format feature adds one impl. The visitors reach it through
/// [`InstalledFormat`](crate::InstalledFormat).
pub trait FormatExt: Format {
    /// Decodes a document's files into a state carrying the format's ext,
    /// boxed.
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[MeshDocumentFile],
    ) -> Result<MeshconvMeshMain>;

    /// Encodes a state as a document's files, writing the loaded file back
    /// exactly when the box holds this format's ext.
    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &Self::WriteOptions,
        main: MeshconvMeshMain,
    ) -> Result<Vec<MeshDocumentFile>>;
}
