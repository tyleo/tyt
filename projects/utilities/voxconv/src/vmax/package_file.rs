use crate::VoxDocumentFile;

/// The bytes of the package file at `path`, or `None` when the document has
/// no such file.
pub fn package_file(files: &[VoxDocumentFile], path: &str) -> Option<Vec<u8>> {
    files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.bytes.clone())
}
