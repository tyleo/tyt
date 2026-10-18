use crate::SdfjAxis;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One rotation of a part about an axis through its pivot: an entry of
/// [`SdfjNode::rotation`](crate::SdfjNode::rotation).
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct SdfjAxisRotation {
    /// The axis of the parent's frame the part rotates about.
    pub axis: SdfjAxis,

    /// How far the part rotates.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub degrees: f64,
}
