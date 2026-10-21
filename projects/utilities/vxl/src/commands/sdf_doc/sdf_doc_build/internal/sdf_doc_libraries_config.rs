use crate::commands::{SdfDocEmbeddedLibrary, SdfDocFileLibrary};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `sdfDoc.build.libraries` section of a `.vxlconfig` layer. The two groups
/// share one namespace.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SdfDocLibrariesConfig {
    /// The libraries whose documents the layer holds.
    pub(crate) embedded: BTreeMap<String, SdfDocEmbeddedLibrary>,

    /// The libraries the layer reads from `.sdfj` or `.sdfjz` files.
    pub(crate) files: BTreeMap<String, SdfDocFileLibrary>,
}
