use crate::{Error, operations::palette::PaletteEditElement};
use branded_id::U32Id;
use std::fmt::Display;
use voxcore::BVoxPalette;

impl Error {
    /// Builds an [`Error::PaletteEdit`] from the element and its reason.
    pub(crate) fn palette_edit(element: PaletteEditElement, reason: impl Display) -> Self {
        Error::PaletteEdit {
            element,
            reason: reason.to_string(),
        }
    }

    /// Wraps `error` in an [`Error::InPalette`] reporting `palette_id`.
    pub(crate) fn in_palette(palette_id: U32Id<BVoxPalette>, error: Error) -> Self {
        Error::InPalette {
            palette_id,
            error: Box::new(error),
        }
    }
}
