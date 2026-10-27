use crate::{Dependencies, Format, ReadFormat, ReadFormatVisitor, Result};
use sdfcore::SdfMain;

/// Decodes a document's bytes into an [`SdfMain`].
pub fn read<D: Dependencies>(
    dependencies: &D,
    format: ReadFormat,
    bytes: &[u8],
) -> Result<SdfMain> {
    format.with(Read {
        dependencies,
        bytes,
    })
}

/// The read of one format.
struct Read<'a, D> {
    dependencies: &'a D,

    bytes: &'a [u8],
}

impl<D: Dependencies> ReadFormatVisitor for Read<'_, D> {
    type Output = Result<SdfMain>;

    fn visit<F: Format>(self) -> Self::Output {
        F::read(self.dependencies, self.bytes)
    }
}
