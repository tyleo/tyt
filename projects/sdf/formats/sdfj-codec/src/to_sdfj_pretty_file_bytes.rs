use crate::{EncodeSdfjJson, Error, Result};
use sdfj::SdfjFile;

/// Serializes `file` through `dependencies` to pretty-printed `.sdfj` JSON
/// bytes with a trailing newline.
pub fn to_sdfj_pretty_file_bytes<D: EncodeSdfjJson>(
    dependencies: &D,
    file: &SdfjFile,
) -> Result<Vec<u8>> {
    let mut bytes = dependencies
        .encode_sdfj_json_pretty(file)
        .map_err(Error::Json)?;

    bytes.push(b'\n');

    Ok(bytes)
}
