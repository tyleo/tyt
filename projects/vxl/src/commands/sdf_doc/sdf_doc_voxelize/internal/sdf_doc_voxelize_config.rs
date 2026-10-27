use crate::commands::SdfDocVoxelizeProfile;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `sdfDoc.voxelize` section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SdfDocVoxelizeConfig {
    pub(crate) profiles: BTreeMap<String, SdfDocVoxelizeProfile>,
}
