use crate::to_sdfj_file;
use sdfcore::SdfMain;
use sdfj_codec::{EncodeSdfjJson, to_sdfj_file_bytes};

/// Writes an [`SdfMain`] to compact `.sdfj` JSON bytes, the bytes form of
/// [`to_sdfj_file()`].
pub fn to_sdfj_bytes<D: EncodeSdfjJson>(dependencies: &D, main: &SdfMain) -> Vec<u8> {
    to_sdfj_file_bytes(dependencies, &to_sdfj_file(main))
        .expect("a validated model holds only finite numbers and unique keys")
}
