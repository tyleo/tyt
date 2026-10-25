use crate::{Dependencies, ReadFormat, Result, WriteFormat};
use sdfcore::SdfMain;
use std::{fmt::Debug, path::Path};

/// An SDF document format, a marker type. Each format feature adds one.
/// [`ReadFormat::with`] and [`WriteFormat::with`] turn a runtime format into
/// the marker.
pub trait Format: 'static {
    /// The writer options. `()` for a format with none.
    type WriteOptions: Clone + Debug + Default + PartialEq;

    /// The short lowercase name, matching the format's feature.
    const NAME: &'static str;

    /// The lowercase file extensions read as this format.
    const EXTENSIONS: &'static [&'static str];

    /// The runtime form of this format.
    fn read_format() -> ReadFormat;

    /// The runtime form of a write in this format with `options`.
    fn write_format(options: Self::WriteOptions) -> WriteFormat;

    /// The extension a document written with `options` takes.
    fn extension(_options: &Self::WriteOptions) -> &'static str {
        Self::EXTENSIONS[0]
    }

    /// Decodes a document's bytes into a model.
    fn read<D: Dependencies>(dependencies: &D, bytes: &[u8]) -> Result<SdfMain>;

    /// Decodes a document's bytes into a model with its fonts loaded. The
    /// document sits in `directory`.
    fn read_with_fonts<D: Dependencies>(
        dependencies: &D,
        bytes: &[u8],
        directory: &Path,
    ) -> Result<SdfMain>;

    /// Encodes a model as a document's bytes.
    fn write<D: Dependencies>(
        dependencies: &D,
        options: &Self::WriteOptions,
        main: &SdfMain,
    ) -> Result<Vec<u8>>;
}
