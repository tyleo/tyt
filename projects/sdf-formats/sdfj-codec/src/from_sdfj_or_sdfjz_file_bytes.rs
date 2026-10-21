use crate::{DecodeSdfjJson, Inflate, Result, from_sdfj_file_bytes, from_sdfjz_file_bytes};
use sdfj::SdfjFile;

/// Decodes either a `.sdfj` (JSON, leading `{`) or `.sdfjz` (zip, leading `PK`)
/// document through `dependencies`, detecting the container by its leading
/// bytes.
pub fn from_sdfj_or_sdfjz_file_bytes<D: DecodeSdfjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<SdfjFile> {
    if bytes.starts_with(b"PK") {
        from_sdfjz_file_bytes(dependencies, bytes)
    } else {
        from_sdfj_file_bytes(dependencies, bytes)
    }
}
