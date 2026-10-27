use crate::commands::MeshDocVoxelizeProfile;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The `meshDoc.voxelize` section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct MeshDocVoxelizeConfig {
    pub(crate) profiles: BTreeMap<String, MeshDocVoxelizeProfile>,
}
