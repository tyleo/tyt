use crate::{VMaxBrushState, VMaxFlag, VMaxValue};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The edit-command record on a history step (`ec`). Per-command payloads are
/// kept as untyped `VMaxValue` (round-trips unchanged).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxEditCommand {
    /// Command samples (e.g. cursor stroke), shape varies by command.
    pub cs: Vec<VMaxValue>,

    /// Mode marker.
    pub mm: i64,

    /// Step token.
    pub ss: String,

    /// Affected region payload.
    pub r: VMaxValue,

    /// Interactive flag.
    pub int: bool,

    /// Brush command payload.
    pub b: VMaxValue,

    /// Brush state ([`VMaxBrushState`](crate::VMaxBrushState)).
    pub bs: VMaxBrushState,

    /// Selection mode; present on some commands.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub sm: Option<i64>,

    /// Selection layer token; present on some commands.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub sl: Option<String>,

    /// Affected-extent payload; present on some commands.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ec: Option<VMaxValue>,

    /// Layer token.
    pub l: String,

    /// Material index.
    pub m: i64,

    /// Command transaction payload (the undo/redo command itself).
    pub comt: VMaxValue,

    /// Mode-detail payload.
    pub md: VMaxValue,

    /// Mirror flag ([`VMaxFlag`](crate::VMaxFlag)); single- or per-axis.
    pub mir: VMaxFlag,

    /// `str` tool flag ([`VMaxFlag`](crate::VMaxFlag)).
    pub str: VMaxFlag,

    /// `strf` tool flag ([`VMaxFlag`](crate::VMaxFlag)).
    pub strf: VMaxFlag,
}
