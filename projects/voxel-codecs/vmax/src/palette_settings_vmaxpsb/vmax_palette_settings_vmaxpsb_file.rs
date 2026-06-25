use crate::{VMaxMaterial, VMaxValue};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Material palette mirroring a `palette*.settings.vmaxpsb` plist.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxPaletteSettingsVmaxpsbFile {
    /// Palette display name.
    pub name: String,

    /// Selectable material slots.
    pub materials: Vec<VMaxMaterial>,

    /// Packed RGBA color table (4 bytes per entry). Voxel Max's color source
    /// when an object's `palette*.png` is absent; palettes that ship a `.png`
    /// omit this key.
    #[cfg_attr(
        feature = "serde",
        serde(with = "serde_bytes", skip_serializing_if = "Vec::is_empty")
    )]
    pub colors: Vec<u8>,

    /// Color indices.
    pub indices: Vec<i64>,

    /// Layer-color usage mask.
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub lc: Vec<u8>,

    /// Palette type tag.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub palette_type: i64,

    /// Global transparency.
    pub transparency: f64,

    /// Reserved counter.
    pub r: i64,

    /// Reserved token.
    pub rt: String,

    /// Comment string.
    pub cmt: String,

    /// Currently selected index.
    pub current: i64,

    /// Alias token.
    pub ali: String,

    /// Per-voxel material assignments some palettes carry; element shape
    /// varies, so each is kept as untyped [`VMaxValue`](crate::VMaxValue)
    /// (round-trips unchanged).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub voxmats: Vec<VMaxValue>,

    /// Layer-settings list some palettes carry; element shape varies, so each
    /// is kept as untyped [`VMaxValue`](crate::VMaxValue) (round-trips
    /// unchanged).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub ls: Vec<VMaxValue>,
}
