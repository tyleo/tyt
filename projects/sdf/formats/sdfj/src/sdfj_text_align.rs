#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Which point of a `text` line sits on u = 0: `start`, `center`, or `end`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum SdfjTextAlign {
    /// The line's middle.
    Center,

    /// The line's end.
    End,

    /// The line's start.
    Start,
}
