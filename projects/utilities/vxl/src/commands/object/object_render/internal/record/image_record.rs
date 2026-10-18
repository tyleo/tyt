use std::slice;
use ty_math::TySrgbU8;

/// One image a run writes, with the views it draws by name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImageRecord {
    /// Several views tiled row by row in order.
    Sheet {
        name: String,

        views: Vec<String>,

        border_width: u32,

        border_color: TySrgbU8,
    },

    /// One view.
    View { name: String, view: String },
}

impl ImageRecord {
    /// The name that suffixes the image's file.
    pub(crate) fn name(&self) -> &str {
        match self {
            ImageRecord::Sheet { name, .. } | ImageRecord::View { name, .. } => name,
        }
    }

    /// The views the image draws, in tile order for a sheet.
    pub(crate) fn views(&self) -> &[String] {
        match self {
            ImageRecord::Sheet { views, .. } => views,
            ImageRecord::View { view, .. } => slice::from_ref(view),
        }
    }
}
