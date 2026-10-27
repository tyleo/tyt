#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One `shades` call: an entry of
/// [`SdfjFile::shades`](crate::SdfjFile::shades).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct SdfjShades {
    /// The material the shades vary.
    pub base: usize,

    /// How many shades the call returns.
    #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
    pub count: f64,

    /// The lightness step between neighboring shades.
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            deserialize_with = "crate::present",
            serialize_with = "crate::finite",
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub spread: Option<f64>,
}
