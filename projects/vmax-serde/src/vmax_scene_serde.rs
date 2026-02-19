use crate::VMaxObjectSerde;
use serde::{Deserialize, Serialize};
use vmax::VMaxScene;

/// Serde-compatible parity type for [`VMaxScene`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMaxSceneSerde {
    pub objects: Vec<VMaxObjectSerde>,
}

impl From<VMaxScene> for VMaxSceneSerde {
    fn from(v: VMaxScene) -> Self {
        Self {
            objects: v.objects.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<VMaxSceneSerde> for VMaxScene {
    fn from(v: VMaxSceneSerde) -> Self {
        Self {
            objects: v.objects.into_iter().map(Into::into).collect(),
        }
    }
}
