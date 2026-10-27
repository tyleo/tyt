use crate::QuantizeProfile;
use serde::Deserialize;
use std::collections::BTreeMap;

/// A quantize command's section of a `.vxlconfig` layer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct QuantizeConfig {
    pub(crate) profiles: BTreeMap<String, QuantizeProfile>,
}
