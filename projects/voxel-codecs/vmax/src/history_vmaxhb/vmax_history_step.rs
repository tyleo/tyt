use crate::{VMaxEditCommand, VMaxTools, VMaxViewCamera};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One step in a Voxel Max history session (`sessions[].steps[]`): the
/// edit-command record plus the viewport cameras and tool state captured for
/// that undo/redo entry.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxHistoryStep {
    /// Edit command ([`VMaxEditCommand`](crate::VMaxEditCommand)).
    pub ec: VMaxEditCommand,

    /// View camera ([`VMaxViewCamera`](crate::VMaxViewCamera)).
    pub vc: VMaxViewCamera,

    /// Focused/framed view camera.
    pub fvc: VMaxViewCamera,

    /// Tool state ([`VMaxTools`](crate::VMaxTools)), the same shape as a
    /// `contents*.vmaxb` object's `tools`.
    pub tc: VMaxTools,
}
