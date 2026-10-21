use crate::{Deflate, EncodeSdfjJson, Result, to_sdfj_file_bytes, wrap_sdfjz};
use sdfj::SdfjFile;

/// Serializes `file` through `dependencies` to a `.sdfjz` zip archive holding
/// one compact `.sdfj` member.
pub fn to_sdfjz_file_bytes<D: EncodeSdfjJson + Deflate>(
    dependencies: &D,
    file: &SdfjFile,
) -> Result<Vec<u8>> {
    Ok(wrap_sdfjz(
        dependencies,
        &to_sdfj_file_bytes(dependencies, file)?,
    ))
}
