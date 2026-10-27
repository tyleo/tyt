use crate::{DecodeSdfjJson, Inflate, Result, from_sdfj_file_bytes, unwrap_sdfjz};
use sdfj::SdfjFile;

/// Decodes `.sdfjz` (zip) bytes by inflating the single `.sdfj` member and
/// decoding it, both through `dependencies`.
pub fn from_sdfjz_file_bytes<D: DecodeSdfjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<SdfjFile> {
    from_sdfj_file_bytes(dependencies, &unwrap_sdfjz(dependencies, bytes)?)
}
