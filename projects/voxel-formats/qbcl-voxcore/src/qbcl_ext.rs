use crate::{QbclExtMetadata, QbclExtNode, QbclExtThumbnail};
#[cfg(feature = "ext")]
use serde::{Deserialize, Serialize};

/// The `qbcl` ext payload stashed on a [`VoxMain`](voxcore::VoxMain):
/// the Qubicle Construction Library `.qbcl` state with no native voxcore home,
/// kept so a file loaded from a `.qbcl` package can be written back exactly.
///
/// Matrix and compound grids become native objects sharing one palette, and the
/// scene tree becomes the hierarchy nodes; this holds the rest, with the
/// per-node entries aligned by index with the hierarchy nodes.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "ext", derive(Deserialize, Serialize))]
pub struct QbclExt {
    /// The version of Qubicle that wrote the file, packed.
    #[cfg_attr(feature = "ext", serde(rename = "program-version"))]
    pub program_version: u32,

    /// The file-format version.
    #[cfg_attr(feature = "ext", serde(rename = "file-version"))]
    pub file_version: u32,

    /// The preview thumbnail from the header.
    #[cfg_attr(feature = "ext", serde(default))]
    pub thumbnail: QbclExtThumbnail,

    /// The seven free-text metadata strings.
    #[cfg_attr(feature = "ext", serde(default))]
    pub metadata: QbclExtMetadata,

    /// The 16-byte header chunk of unconfirmed purpose, preserved verbatim.
    pub guid: [u8; 16],

    /// Per scene-node provenance, aligned by index with the hierarchy nodes.
    #[cfg_attr(feature = "ext", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub nodes: Vec<QbclExtNode>,
}
