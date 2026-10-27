#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// How an arc closes its ends.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SdfjCaps {
    /// `flat`.
    Flat,

    /// `round`.
    Round,
}
