use crate::{
    VMaxCamera, VMaxDiscarded, VMaxEmptyList, VMaxLegacyChunkVoxels, VMaxSnapshot, VMaxTools,
    VMaxValue,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Voxel object payload of a `contents*.vmaxb` binary plist: the geometry,
/// camera, extent, and work area Voxel Max renders the object from. The brush
/// palette, editor state, is named, read, and never written, since Voxel Max
/// opens an object without it. An unknown key is an error.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxContentsVmaxbFile {
    /// Per-chunk voxel snapshots: the baked geometry edit log.
    #[cfg_attr(feature = "serde", serde(default))]
    pub snapshots: Vec<VMaxSnapshot>,

    /// Object content UUID.
    #[cfg_attr(feature = "serde", serde(default))]
    pub uuid: String,

    /// Codable version.
    #[cfg_attr(feature = "serde", serde(default))]
    pub v: i64,

    /// The tool settings, holding the object's work area. Absent, the work
    /// area spans the whole `eo` extent.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub tools: Option<VMaxTools>,

    /// The brush palette, editor state.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing))]
    pub brush: Option<VMaxDiscarded>,

    /// A list current Voxel Max writes empty, meaning unknown.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing))]
    pub bbr: Option<VMaxEmptyList>,

    /// Per-object camera. Required: Voxel Max opens no package holding a
    /// contents file without one.
    pub cam: VMaxCamera,

    /// Embedded palette dictionary some objects carry (e.g. MagicaVoxel
    /// exports); shape differs from a `.vmaxpsb`, so kept as untyped
    /// [`VMaxValue`] (round-trips unchanged).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub pal: Option<VMaxValue>,

    /// The order of the object's voxel extent, `2^eo` voxels on a side. Voxel
    /// Max reads 5 to 9 and takes 8 for anything else.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub eo: Option<i64>,

    /// The chunk records of a contents file older than version 4, as stored:
    /// 16 bytes per chunk, little-endian 32-bit integers for a voxel origin
    /// inside the chunk and, from version 1, a fourth that is 1 for a chunk of
    /// two-byte voxels.
    #[cfg_attr(
        feature = "serde",
        serde(default, with = "serde_bytes", skip_serializing_if = "Vec::is_empty")
    )]
    pub chunks: Vec<u8>,

    /// The voxels of each legacy chunk, in `chunks` order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub voxels: Vec<VMaxLegacyChunkVoxels>,
}
