/// One file of a voxel document: its path relative to the document and its
/// bytes. A single-file format's document has one entry with an empty path.
/// A package's entries carry their package-relative paths, such as
/// `QuickLook/Thumbnail.png`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoxDocumentFile {
    /// The document-relative path, empty for a single-file document.
    pub path: String,

    /// The file's bytes.
    pub bytes: Vec<u8>,
}

impl VoxDocumentFile {
    /// A package file at `path`.
    pub fn new(path: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            bytes,
        }
    }

    /// The one file of a single-file document.
    pub fn single(bytes: Vec<u8>) -> Self {
        Self {
            path: String::new(),
            bytes,
        }
    }
}
