use serde::Deserialize;
use std::path::PathBuf;

/// A library a `.vxlconfig` layer reads from an `.sdfj` or `.sdfjz` file.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SdfDocFileLibrary {
    /// What the library holds.
    #[serde(default)]
    pub(crate) description: Option<String>,

    /// The `.sdfj` file's path relative to the directory of the `.vxlconfig`
    /// holding it.
    pub(crate) path: PathBuf,
}
