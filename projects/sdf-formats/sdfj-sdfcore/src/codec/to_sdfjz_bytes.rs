use crate::to_sdfj_file;
use sdfcore::SdfMain;
use sdfj_codec::{Deflate, EncodeSdfjJson, to_sdfjz_file_bytes};

/// Writes an [`SdfMain`] to a `.sdfjz` zip archive holding one compact
/// `.sdfj` member, the bytes form of [`to_sdfj_file()`].
pub fn to_sdfjz_bytes<D: EncodeSdfjJson + Deflate>(dependencies: &D, main: &SdfMain) -> Vec<u8> {
    to_sdfjz_file_bytes(dependencies, &to_sdfj_file(main))
        .expect("a validated model holds only finite numbers and unique keys")
}
