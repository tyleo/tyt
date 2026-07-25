use crate::{VoxjEditState, VoxjMap, VoxjRuntimeState};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize};

/// The body of a Voxel Json document.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct VoxjMain {
    /// The runtime scene: objects, palettes, hierarchy, and roots.
    pub runtime_state: VoxjRuntimeState,

    /// Optional editor state, aligned by index with the runtime objects.
    /// Absent in fully-runtime documents.
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub edit_state: Option<VoxjEditState>,

    /// Optional namespace for user-defined extensions, conventionally
    /// vendor-keyed. The core format assigns it no meaning.
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub ext: Option<VoxjMap>,
}

/// Deserializes a present optional field, so an explicit `null` rejects.
#[cfg(feature = "serde")]
fn present<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    T::deserialize(deserializer).map(Some)
}
