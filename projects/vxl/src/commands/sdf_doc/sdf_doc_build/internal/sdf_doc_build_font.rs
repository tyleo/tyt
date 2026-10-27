use serde::Deserialize;
use std::path::PathBuf;

/// A font a build profile's `fonts` holds, which `--font` mirrors.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum SdfDocBuildFont {
    /// A font file at `path`, relative to the directory of the `.vxlconfig`
    /// holding it.
    File { path: PathBuf },

    /// The installed face whose family or full name matches `font`.
    Installed { font: String },
}
