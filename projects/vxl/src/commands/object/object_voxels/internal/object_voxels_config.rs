use crate::QuantizeConfig;
use serde::Deserialize;

/// The `object.voxels` section of a `.vxlconfig` layer, one entry per
/// `object voxels` command that reads configuration.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ObjectVoxelsConfig {
    pub(crate) quantize: QuantizeConfig,
}
