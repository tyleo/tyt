/// Where a written document's images go.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GltfImageStorage {
    /// Where each image reads from. An image over one of the document's
    /// files is referenced by the file's name. An image over its own bytes
    /// embeds where the ext recorded it was loaded from, or in a buffer view
    /// when it recorded nothing.
    #[default]
    AsLoaded,

    /// Every image embeds in the document: a buffer view, or a data URI for
    /// an image loaded from one. An image over one of the document's files
    /// embeds a copy, and the file still lands beside the document.
    Embedded,

    /// Every image lands as a loose file beside the document: an image over
    /// one of the document's files under that file's name, and an image over
    /// its own bytes under its name, else its index.
    Loose,
}
