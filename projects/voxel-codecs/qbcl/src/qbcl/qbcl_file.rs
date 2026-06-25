use crate::qbcl::{QbclMetadata, QbclNode, QbclThumbnail};

/// A Qubicle Construction Library `.qbcl` file: header, preview thumbnail,
/// metadata, and scene tree.
#[derive(Clone, Debug, PartialEq)]
pub struct QbclFile {
    /// The version of Qubicle that wrote the file, a packed
    /// `major.minor.release.build`. Cosmetic; preserved as stored.
    pub program_version: u32,

    /// The file-format version; this crate reads and writes `2`.
    pub file_version: u32,

    /// The preview image embedded in the header.
    pub thumbnail: QbclThumbnail,

    /// The seven free-text metadata strings.
    pub metadata: QbclMetadata,

    /// A 16-byte header chunk of unconfirmed purpose (a GUID or a pair of
    /// timestamps), preserved verbatim so the file round-trips.
    pub guid: [u8; 16],

    /// The scene-tree root, conventionally a [`QbclNodeBody::Model`].
    ///
    /// [`QbclNodeBody::Model`]: crate::qbcl::QbclNodeBody::Model
    pub root: QbclNode,
}

impl Default for QbclFile {
    fn default() -> Self {
        Self {
            program_version: 0,
            file_version: 2,
            thumbnail: QbclThumbnail::default(),
            metadata: QbclMetadata::default(),
            guid: [0; 16],
            root: QbclNode::default(),
        }
    }
}
