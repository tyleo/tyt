use crate::{
    VoxMap,
    ext::{Result, VoxExtEntryCodec},
};

/// Decodes a format's ext from its entry in `block`, or `None` when the block
/// holds nothing under the format's key and so belongs to another format.
pub fn decode_entry<E: VoxExtEntryCodec>(block: &VoxMap) -> Result<Option<E>> {
    block
        .0
        .iter()
        .find(|(key, _)| key == E::KEY)
        .map(|(_, value)| E::from_vox_ext_entry(value))
        .transpose()
}

#[cfg(test)]
mod tests {
    use crate::{
        VoxMap, VoxValue,
        ext::{Error, Result, VoxExtBlockCodec, VoxExtEntryCodec, decode_entry, encode_entry},
    };

    #[derive(Debug, PartialEq)]
    struct Count(u32);

    impl VoxExtEntryCodec for Count {
        const KEY: &'static str = "count";

        fn to_vox_ext_entry(&self) -> Result<VoxValue> {
            Ok(VoxValue::Number(self.0 as f64))
        }

        fn from_vox_ext_entry(value: &VoxValue) -> Result<Self> {
            match value {
                VoxValue::Number(number) => Ok(Count(*number as u32)),
                _ => Err(Error::Invalid("count must be a number".to_owned())),
            }
        }
    }

    #[test]
    fn an_entry_round_trips_through_its_one_entry_block() {
        let block = encode_entry(&Count(7)).unwrap();

        assert_eq!(
            block,
            VoxMap(vec![("count".to_owned(), VoxValue::Number(7.0))])
        );

        assert_eq!(decode_entry::<Count>(&block).unwrap(), Some(Count(7)));
    }

    #[test]
    fn a_foreign_block_decodes_to_none_and_a_bad_entry_errors() {
        let foreign = VoxMap(vec![("other".to_owned(), VoxValue::Number(7.0))]);

        assert_eq!(decode_entry::<Count>(&foreign).unwrap(), None);

        let bad = VoxMap(vec![("count".to_owned(), VoxValue::Null)]);

        assert!(decode_entry::<Count>(&bad).is_err());
    }

    #[test]
    fn an_optional_ext_loads_through_its_entry() {
        let block = encode_entry(&Count(3)).unwrap();

        assert_eq!(
            Option::<Count>::from_vox_ext_block(Some(&block)).unwrap(),
            Some(Count(3))
        );

        let foreign = VoxMap(vec![("other".to_owned(), VoxValue::Null)]);

        assert_eq!(
            Option::<Count>::from_vox_ext_block(Some(&foreign)).unwrap(),
            None
        );

        assert_eq!(Option::<Count>::from_vox_ext_block(None).unwrap(), None);
    }
}
