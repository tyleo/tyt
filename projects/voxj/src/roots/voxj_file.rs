use crate::{VoxjBackend, VoxjMain};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The root of a Voxel Json document, generic over the object representation
/// (see [`VoxjBackend`]).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "Backend::Object: Serialize",
        deserialize = "Backend::Object: Deserialize<'de>"
    ))
)]
pub struct VoxjFile<Backend: VoxjBackend> {
    pub version: u32,

    pub main: VoxjMain<Backend>,
}
