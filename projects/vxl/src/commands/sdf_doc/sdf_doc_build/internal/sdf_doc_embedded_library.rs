use sdfj::SdfjFile;
use serde::Deserialize;

/// A library a `.vxlconfig` layer holds whole.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SdfDocEmbeddedLibrary {
    /// What the library holds.
    #[serde(default)]
    pub(crate) description: Option<String>,

    /// The library's `.sdfj` document.
    pub(crate) document: SdfjFile,
}
