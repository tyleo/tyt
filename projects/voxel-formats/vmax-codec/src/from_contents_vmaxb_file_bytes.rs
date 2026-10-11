use crate::{DecodeVMaxPlist, DecompressLzfse, Error, Result, decompress_lzfse_or_raw};
use vmax::VMaxContentsVmaxbFile;

/// Decodes `contents*.vmaxb` bytes (an LZFSE-framed binary plist) into a
/// [`VMaxContentsVmaxbFile`] through `dependencies`.
pub fn from_contents_vmaxb_file_bytes<D: DecompressLzfse + DecodeVMaxPlist>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VMaxContentsVmaxbFile> {
    let plist_bytes = decompress_lzfse_or_raw(dependencies, bytes);
    dependencies
        .decode_contents_vmaxb(&plist_bytes)
        .map_err(Error::Plist)
}

#[cfg(test)]
mod tests {
    use plist::{Dictionary, Value, from_bytes};
    use vmax::VMaxStats;

    /// Current Voxel Max adds derived position-sum caches to chunk stats.
    /// They must not prevent the authoritative bounds and count from reading.
    #[test]
    fn reads_stats_with_current_position_sum_caches() {
        let mut dictionary = Dictionary::new();
        dictionary.insert(
            "min".to_owned(),
            Value::Array([1, 2, 3, 0].map(|v| Value::Integer(v.into())).to_vec()),
        );
        dictionary.insert("count".to_owned(), Value::Integer(1.into()));
        for key in ["psum", "spsum"] {
            dictionary.insert(
                key.to_owned(),
                Value::Array([1, 2, 3].map(|v| Value::Integer(v.into())).to_vec()),
            );
        }
        dictionary.insert("psv".to_owned(), Value::Integer(1.into()));
        let mut bytes = Vec::new();
        Value::Dictionary(dictionary)
            .to_writer_binary(&mut bytes)
            .unwrap();
        let stats = from_bytes::<VMaxStats>(&bytes).unwrap();
        assert_eq!(stats.min, [1, 2, 3, 0]);
        assert_eq!(stats.count, 1);
    }
}
