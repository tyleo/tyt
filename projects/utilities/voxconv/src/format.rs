use crate::{Dependencies, ReadFormat, Result, VoxDocumentFile, WriteFormat};
use std::fmt::Debug;
use voxcore::{VoxMain, check::VoxCheck};

/// A voxel document format: the facts that pick it and the bare read and
/// write over its bridge. A format is a marker type. Each format feature
/// adds one. [`ReadFormat::with`] and [`WriteFormat::with`] turn a runtime
/// format into the marker.
pub trait Format: 'static {
    /// The writer options. `()` for a format with none.
    type WriteOptions: Clone + Debug + Default + PartialEq;

    /// The short lowercase name, as the format features spell it.
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

    /// Decodes a document's files into a bare state. The format's ext drops.
    fn read<D: Dependencies>(dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>>;

    /// Encodes a bare state as a document's files synthesized from its scene.
    fn write<D: Dependencies>(
        dependencies: &D,
        options: &Self::WriteOptions,
        state: VoxMain<()>,
    ) -> Result<Vec<VoxDocumentFile>>;

    /// The format's spec checks over a document's files that decode. A
    /// format with no spec checks reports none.
    fn check<D: Dependencies>(
        _dependencies: &D,
        _files: &[VoxDocumentFile],
    ) -> Result<Vec<VoxCheck>> {
        Ok(Vec::new())
    }
}
