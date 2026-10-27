use crate::{Result, from_sdfj_file};
use sdfcore::SdfMain;
use sdfj_codec::{DecodeSdfjJson, Inflate, from_sdfj_or_sdfjz_file_bytes};

/// Loads a `.sdfj` or `.sdfjz` document into an [`SdfMain`], the bytes form
/// of [`from_sdfj_file()`]. The container form is detected from the leading
/// bytes.
pub fn from_sdfj_bytes<D: DecodeSdfjJson + Inflate>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<SdfMain> {
    from_sdfj_file(&from_sdfj_or_sdfjz_file_bytes(dependencies, bytes)?)
}

#[cfg(test)]
mod tests {
    use crate::{
        CHAIR_SDFJ, Error,
        codec::{from_sdfj_bytes, to_sdfj_bytes, to_sdfj_pretty_bytes, to_sdfjz_bytes},
    };
    use sdfj_codec::DependenciesImpl;

    #[test]
    fn compact_pretty_and_zip_bytes_round_trip() {
        let main = from_sdfj_bytes(&DependenciesImpl, CHAIR_SDFJ.as_bytes()).unwrap();

        for bytes in [
            to_sdfj_bytes(&DependenciesImpl, &main),
            to_sdfj_pretty_bytes(&DependenciesImpl, &main),
            to_sdfjz_bytes(&DependenciesImpl, &main),
        ] {
            assert_eq!(from_sdfj_bytes(&DependenciesImpl, &bytes).unwrap(), main);
        }
    }

    #[test]
    fn undecodable_bytes_error_in_the_codec() {
        assert!(matches!(
            from_sdfj_bytes(&DependenciesImpl, b"not a document"),
            Err(Error::Codec(_))
        ));
    }
}
