use crate::BMeshFile;
use branded_id::U32Id;

/// Where a [`MeshImage`](crate::MeshImage) reads its encoded bytes from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeshImageSource {
    /// The image's own bytes, which a container embeds.
    Bytes(Vec<u8>),

    /// One of the document's files, which a writer references by name.
    File(U32Id<BMeshFile>),
}
