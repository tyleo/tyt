#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A font a `text` shape reads: an entry of
/// [`SdfjFile::fonts`](crate::SdfjFile::fonts).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        tag = "kind",
        rename_all = "camelCase",
        rename_all_fields = "camelCase",
        deny_unknown_fields
    )
)]
pub enum SdfjFont {
    /// A font file at `path`, relative to the document's directory. A font
    /// collection gives its first face.
    File { name: String, path: String },

    /// The installed face whose family or full name matches `font`, ignoring
    /// case. A family picks its regular face.
    Installed { name: String, font: String },
}
