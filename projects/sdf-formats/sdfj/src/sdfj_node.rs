use crate::SdfjAxisRotation;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A part's place in the hierarchy: an entry of
/// [`SdfjFile::nodes`](crate::SdfjFile::nodes).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct SdfjNode {
    /// The part's name.
    pub name: String,

    /// The joint the part turns about, in meters.
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            deserialize_with = "crate::present",
            serialize_with = "crate::finite",
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub pivot: Option<[f64; 3]>,

    /// How far the part moves, in meters.
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            deserialize_with = "crate::present",
            serialize_with = "crate::finite",
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub offset: Option<[f64; 3]>,

    /// The rotations of the part about its pivot, in order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub rotation: Vec<SdfjAxisRotation>,

    /// Indices into [`SdfjFile::objects`](crate::SdfjFile::objects).
    pub child_objects: Vec<usize>,

    /// Indices into [`SdfjFile::nodes`](crate::SdfjFile::nodes), each before
    /// this node.
    pub child_nodes: Vec<usize>,
}
