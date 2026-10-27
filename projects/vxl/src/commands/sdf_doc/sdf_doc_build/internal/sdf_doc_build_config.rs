use crate::commands::{SdfDocBuildProfile, SdfDocLibrariesConfig};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `sdfDoc.build` section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SdfDocBuildConfig {
    pub(crate) libraries: SdfDocLibrariesConfig,

    pub(crate) profiles: BTreeMap<String, SdfDocBuildProfile>,
}
