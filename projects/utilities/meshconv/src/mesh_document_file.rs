use crate::{Error, Result};

/// One file of a mesh document. The primary file sits at the empty path. A
/// loose file sits at the relative URI the primary references it by, such
/// as `skin.png`. A package's entries carry their package-relative paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshDocumentFile {
    /// The document-relative path, empty for the primary file.
    pub path: String,

    /// The file's bytes.
    pub bytes: Vec<u8>,
}

impl MeshDocumentFile {
    /// A file at `path`.
    pub fn new(path: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            bytes,
        }
    }

    /// The primary file of a document.
    pub fn primary(bytes: Vec<u8>) -> Self {
        Self {
            path: String::new(),
            bytes,
        }
    }

    /// The bytes of the primary file, the one at the empty path. No such
    /// file, or more than one, is an error.
    pub fn primary_bytes(files: &[MeshDocumentFile]) -> Result<&[u8]> {
        let mut primaries = files.iter().filter(|file| file.path.is_empty());

        match (primaries.next(), primaries.next()) {
            (Some(file), None) => Ok(&file.bytes),
            (None, _) => Err(Error::Files(
                "a document holds one primary file at the empty path, and this holds none"
                    .to_owned(),
            )),
            (Some(_), Some(_)) => Err(Error::Files(
                "a document holds one primary file at the empty path, and this holds several"
                    .to_owned(),
            )),
        }
    }

    /// The files beside the primary, each as its path and bytes.
    pub fn loose_files(files: &[MeshDocumentFile]) -> impl Iterator<Item = (&str, &[u8])> {
        files
            .iter()
            .filter(|file| !file.path.is_empty())
            .map(|file| (file.path.as_str(), file.bytes.as_slice()))
    }
}
