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
    use plist::{self, Dictionary, Value, from_bytes};
    use vmax::{
        VMaxContentsVmaxbFile, VMaxDiscarded, VMaxEmptyList, VMaxStats, VMaxTools, VMaxViewBox,
    };

    fn integers(values: &[i64]) -> Value {
        Value::Array(values.iter().map(|&v| Value::Integer(v.into())).collect())
    }

    fn binary(dictionary: Dictionary) -> Vec<u8> {
        let mut bytes = Vec::new();

        Value::Dictionary(dictionary)
            .to_writer_binary(&mut bytes)
            .unwrap();

        bytes
    }

    /// A contents dictionary holding only its required per-object camera.
    fn with_camera() -> Dictionary {
        let mut dictionary = Dictionary::new();

        dictionary.insert("cam".to_owned(), Value::Dictionary(Dictionary::new()));

        dictionary
    }

    /// Current Voxel Max adds derived position-sum caches to chunk stats,
    /// which read beside the authoritative bounds and count.
    #[test]
    fn reads_stats_with_current_position_sum_caches() {
        let mut dictionary = Dictionary::new();

        dictionary.insert("min".to_owned(), integers(&[1, 2, 3, 0]));

        dictionary.insert("count".to_owned(), Value::Integer(1.into()));

        dictionary.insert("psum".to_owned(), integers(&[1, 2, 3]));

        dictionary.insert("spsum".to_owned(), integers(&[4, 5, 6]));

        dictionary.insert("psv".to_owned(), Value::Integer(1.into()));

        let stats = from_bytes::<VMaxStats>(&binary(dictionary)).unwrap();

        assert_eq!(stats.min, [1, 2, 3, 0]);

        assert_eq!(stats.count, 1);

        assert_eq!(stats.psum, Some(vec![1, 2, 3]));

        assert_eq!(stats.spsum, Some(vec![4, 5, 6]));

        assert_eq!(stats.psv, Some(1));
    }

    /// Current Voxel Max caches the voxels inside the work area, here from a
    /// save whose work area hides 4 of 9 voxels.
    #[test]
    fn reads_stats_with_current_work_area_caches() {
        let mut dictionary = Dictionary::new();

        dictionary.insert("min".to_owned(), integers(&[0, 8192, 0, 8192]));

        dictionary.insert("count".to_owned(), Value::Integer(9.into()));

        dictionary.insert("emax".to_owned(), integers(&[72, 9362, 0, 9434]));

        dictionary.insert("ecnt".to_owned(), Value::Integer(5.into()));

        let stats = from_bytes::<VMaxStats>(&binary(dictionary)).unwrap();

        assert_eq!(stats.emax, Some(vec![72, 9362, 0, 9434]));

        assert_eq!(stats.ecnt, Some(5));
    }

    /// A stats key the model does not name is an error, so a newer Voxel Max
    /// key surfaces instead of reading silently.
    #[test]
    fn rejects_an_unknown_stats_key() {
        let mut dictionary = Dictionary::new();

        dictionary.insert("cuont".to_owned(), Value::Integer(1.into()));

        let error = from_bytes::<VMaxStats>(&binary(dictionary)).unwrap_err();

        assert!(error.to_string().contains("cuont"), "{error}");
    }

    /// The brush palette is editor state: it reads, keeps nothing, and is
    /// never written.
    #[test]
    fn reads_and_discards_the_brush_palette() {
        let mut brush = Dictionary::new();

        brush.insert("name".to_owned(), Value::String("Default".to_owned()));

        let mut dictionary = with_camera();

        dictionary.insert("brush".to_owned(), Value::Dictionary(brush));

        let contents = from_bytes::<VMaxContentsVmaxbFile>(&binary(dictionary)).unwrap();

        assert_eq!(contents.brush, Some(VMaxDiscarded));

        let mut written = Vec::new();

        plist::to_writer_binary(&mut written, &contents).unwrap();

        let written = from_bytes::<Dictionary>(&written).unwrap();

        assert!(!written.contains_key("brush"));
    }

    /// The `bbr` list current Voxel Max writes empty reads, and is never
    /// written.
    #[test]
    fn reads_an_empty_bbr_list() {
        let mut dictionary = with_camera();

        dictionary.insert("bbr".to_owned(), Value::Array(Vec::new()));

        let contents = from_bytes::<VMaxContentsVmaxbFile>(&binary(dictionary)).unwrap();

        assert_eq!(contents.bbr, Some(VMaxEmptyList));

        let mut written = Vec::new();

        plist::to_writer_binary(&mut written, &contents).unwrap();

        let written = from_bytes::<Dictionary>(&written).unwrap();

        assert!(!written.contains_key("bbr"));
    }

    /// A `bbr` list with entries is an error until the model knows what they
    /// hold.
    #[test]
    fn rejects_a_bbr_list_with_entries() {
        let mut dictionary = with_camera();

        dictionary.insert("bbr".to_owned(), integers(&[1]));

        let error = from_bytes::<VMaxContentsVmaxbFile>(&binary(dictionary)).unwrap_err();

        assert!(error.to_string().contains("not empty"), "{error}");
    }

    /// A contents file without its per-object camera is an error, since Voxel
    /// Max opens no package holding one.
    #[test]
    fn rejects_a_contents_file_without_a_camera() {
        let error = from_bytes::<VMaxContentsVmaxbFile>(&binary(Dictionary::new())).unwrap_err();

        assert!(error.to_string().contains("cam"), "{error}");
    }

    /// Fresh tool settings as a plist dictionary, around a small work area.
    fn fresh_tools() -> Dictionary {
        let tools = VMaxTools::fresh(VMaxViewBox {
            min: [8, 10, 0],
            max: [23, 21, 7],
            flat: None,
        });

        let mut bytes = Vec::new();

        plist::to_writer_binary(&mut bytes, &tools).unwrap();

        from_bytes::<Dictionary>(&bytes).unwrap()
    }

    /// The tool settings read back exactly as written, work area included.
    #[test]
    fn reads_the_tool_settings_it_writes() {
        let dictionary = fresh_tools();

        assert_eq!(dictionary.len(), 22);

        let tools = from_bytes::<VMaxTools>(&binary(dictionary)).unwrap();

        assert_eq!(tools.vp.max, [23, 21, 7]);

        assert_eq!(tools.mi, 35);
    }

    /// Voxel Max opens no object whose tool settings lack part of the tool
    /// context, so a missing setting is an error.
    #[test]
    fn rejects_tool_settings_missing_a_setting() {
        let mut dictionary = fresh_tools();

        dictionary.remove("mi");

        let error = from_bytes::<VMaxTools>(&binary(dictionary)).unwrap_err();

        assert!(error.to_string().contains("mi"), "{error}");
    }

    /// A tool setting the model does not name is an error.
    #[test]
    fn rejects_an_unknown_tool_setting() {
        let mut dictionary = fresh_tools();

        dictionary.insert("zz".to_owned(), Value::Integer(1.into()));

        let error = from_bytes::<VMaxTools>(&binary(dictionary)).unwrap_err();

        assert!(error.to_string().contains("zz"), "{error}");
    }

    /// A contents key the model does not name is an error.
    #[test]
    fn rejects_an_unknown_contents_key() {
        let mut dictionary = with_camera();

        dictionary.insert("bld".to_owned(), Value::Integer(1.into()));

        let error = from_bytes::<VMaxContentsVmaxbFile>(&binary(dictionary)).unwrap_err();

        assert!(error.to_string().contains("bld"), "{error}");
    }
}
