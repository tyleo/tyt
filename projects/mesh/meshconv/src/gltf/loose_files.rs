use crate::MeshDocumentFile;
use std::collections::BTreeMap;

/// The files beside the primary, keyed by their relative paths.
pub fn loose_files(files: &[MeshDocumentFile]) -> BTreeMap<String, Vec<u8>> {
    MeshDocumentFile::loose_files(files)
        .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()))
        .collect()
}
