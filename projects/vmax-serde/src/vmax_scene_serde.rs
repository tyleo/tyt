use crate::{VMaxGroupSerde, VMaxObjectSerde};
use serde::{Deserialize, Serialize};
use vmax::VMaxScene;

/// Serde-compatible parity type for [`VMaxScene`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMaxSceneSerde {
    #[serde(default)]
    pub groups: Vec<VMaxGroupSerde>,
    pub objects: Vec<VMaxObjectSerde>,
}

impl From<VMaxScene> for VMaxSceneSerde {
    fn from(v: VMaxScene) -> Self {
        Self {
            groups: v.groups.into_iter().map(Into::into).collect(),
            objects: v.objects.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<VMaxSceneSerde> for VMaxScene {
    fn from(v: VMaxSceneSerde) -> Self {
        Self {
            groups: v.groups.into_iter().map(Into::into).collect(),
            objects: v.objects.into_iter().map(Into::into).collect(),
        }
    }
}
