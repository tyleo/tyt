#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A part's steps: an entry of [`SdfjFile::objects`](crate::SdfjFile::objects).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct SdfjObject {
    /// The part's name.
    pub name: String,

    /// Indices into [`SdfjFile::steps`](crate::SdfjFile::steps), in list
    /// order.
    pub steps: Vec<usize>,
}
