use crate::{Error, Result, VoxDocumentFile};

/// The bytes of a single-file document. More or fewer than one file is an
/// error.
pub fn single_file_bytes(files: &[VoxDocumentFile]) -> Result<&[u8]> {
    match files {
        [file] => Ok(&file.bytes),
        _ => Err(Error::Files(format!(
            "a single-file document holds one file, not {}",
            files.len()
        ))),
    }
}
