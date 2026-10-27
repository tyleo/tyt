use crate::CliValue;

/// What an image draws.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageKind {
    /// Several views tiled into one image.
    Sheet,

    /// One view.
    View,
}

impl CliValue for ImageKind {
    const VARIANTS: &'static [Self] = &[ImageKind::Sheet, ImageKind::View];

    fn name(self) -> &'static str {
        match self {
            ImageKind::Sheet => "sheet",
            ImageKind::View => "view",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ImageKind::Sheet => "Several views tiled row by row with a border between them",
            ImageKind::View => "One view",
        }
    }
}
