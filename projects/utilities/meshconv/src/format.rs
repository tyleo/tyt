use crate::{Dependencies, MeshDocumentFile, ReadFormat, Result, WriteFormat};
use meshdoc::{MeshMain, check::MeshCheck};
use std::fmt::Debug;

/// A mesh document format, a marker type. Each format feature adds one.
/// [`ReadFormat::with`] and [`WriteFormat::with`] turn a runtime format into
/// the marker.
pub trait Format: 'static {
    /// The writer options. `()` for a format with none.
    type WriteOptions: Clone + Debug + Default + PartialEq;

    /// The short lowercase name, matching the format's feature.
    const NAME: &'static str;

    /// The lowercase file extensions read as this format.
    const EXTENSIONS: &'static [&'static str];

    /// Whether a document is a package directory.
    const PACKAGE: bool = false;

    /// The runtime form of this format.
    fn read_format() -> ReadFormat;

    /// The runtime form of a write in this format with `options`.
    fn write_format(options: Self::WriteOptions) -> WriteFormat;

    /// The extension a document written with `options` takes.
    fn extension(_options: &Self::WriteOptions) -> &'static str {
        Self::EXTENSIONS[0]
    }

    /// The relative paths of the loose files a primary's bytes reference,
    /// which sit beside it. A format whose documents are one file lists
    /// none.
    fn loose_paths(_primary: &[u8]) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Decodes a document's files into a bare state. The format's ext drops.
    fn read<D: Dependencies>(dependencies: &D, files: &[MeshDocumentFile]) -> Result<MeshMain<()>>;

    /// Encodes a bare state as a document's files.
    fn write<D: Dependencies>(
        dependencies: &D,
        options: &Self::WriteOptions,
        main: MeshMain<()>,
    ) -> Result<Vec<MeshDocumentFile>>;

    /// The format's spec checks over a document's files that decode. A
    /// format with no spec checks reports none.
    fn check<D: Dependencies>(
        _dependencies: &D,
        _files: &[MeshDocumentFile],
    ) -> Result<Vec<MeshCheck>> {
        Ok(Vec::new())
    }
}
