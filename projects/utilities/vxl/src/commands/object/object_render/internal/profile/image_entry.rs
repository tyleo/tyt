use crate::{
    NamedCliValue,
    commands::{ImageKind, SrgbColor},
};
use serde::Deserialize;

/// A profile's image, keyed by the name that suffixes its file.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ImageEntry {
    /// Mirrors `--image-kind`. Defaults to `view`.
    pub(crate) kind: Option<NamedCliValue<ImageKind>>,

    /// Mirrors `--image-view`, the view a `view` image draws. Defaults to the
    /// image's name.
    pub(crate) view: Option<String>,

    /// Mirrors `--image-views` per entry, the views a `sheet` tiles in order.
    pub(crate) views: Option<Vec<String>>,

    /// Mirrors `--image-border-width`, in pixels. Defaults to `8`.
    pub(crate) border_width: Option<u32>,

    /// Mirrors `--image-border-color`. Defaults to `#808080`.
    pub(crate) border_color: Option<SrgbColor>,
}
