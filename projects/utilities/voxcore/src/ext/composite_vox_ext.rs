use crate::{
    BVoxVoxel, VoxMap,
    ext::{Error, Result, VoxExt, VoxExtEntryCodec, decode_entry},
};
use branded_id::U32Id;
use std::any::Any;

/// A block's entries, each as a boxed ext, in the block's order. The
/// composite starts from a [`VoxMap`] with every entry verbatim as a
/// one-entry map. [`decode`](Self::decode) turns the entry a format owns into
/// that format's ext. Every hook forwards to every entry, so a decoded ext
/// follows a mutation while a verbatim entry cannot.
/// [`to_vox_ext`](VoxExt::to_vox_ext) merges the entries back into one block
/// and errors when two share a key.
#[derive(Debug, Default)]
pub struct CompositeVoxExt {
    entries: Vec<Box<dyn VoxExt>>,
}

impl CompositeVoxExt {
    /// Decodes the entry `E` owns into `E`, in place. An entry under `E`'s
    /// key that does not decode is an error. A composite with no such entry
    /// is unchanged.
    pub fn decode<E: VoxExt + VoxExtEntryCodec>(mut self) -> Result<Self> {
        for entry in &mut self.entries {
            let Some(block) = entry.as_any().downcast_ref::<VoxMap>() else {
                continue;
            };

            let Some(decoded) = decode_entry::<E>(block)? else {
                continue;
            };

            *entry = Box::new(decoded);
        }

        Ok(self)
    }
}

/// Every entry verbatim.
impl From<VoxMap> for CompositeVoxExt {
    fn from(block: VoxMap) -> Self {
        Self {
            entries: block
                .0
                .into_iter()
                .map(|entry| Box::new(VoxMap(vec![entry])) as Box<dyn VoxExt>)
                .collect(),
        }
    }
}

impl Clone for CompositeVoxExt {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.iter().map(|entry| entry.clone_box()).collect(),
        }
    }
}

impl VoxExt for CompositeVoxExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        let mut block = VoxMap::default();

        for entry in &self.entries {
            for (key, value) in entry.to_vox_ext()?.0 {
                if block.0.iter().any(|(taken, _)| *taken == key) {
                    return Err(Error::Invalid(format!("the ext block holds `{key}` twice")));
                }

                block.0.push((key, value));
            }
        }

        Ok(block)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }

    fn hierarchy_node_did_retain(&mut self, index: usize) {
        for entry in &mut self.entries {
            entry.hierarchy_node_did_retain(index);
        }
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        for entry in &mut self.entries {
            entry.hierarchy_node_will_release(index);
        }
    }

    fn object_did_retain(&mut self, index: usize) {
        for entry in &mut self.entries {
            entry.object_did_retain(index);
        }
    }

    fn object_will_release(&mut self, index: usize) {
        for entry in &mut self.entries {
            entry.object_will_release(index);
        }
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        for entry in &mut self.entries {
            entry.object_did_move(from, to);
        }
    }

    fn palette_did_retain(&mut self, index: usize) {
        for entry in &mut self.entries {
            entry.palette_did_retain(index);
        }
    }

    fn palette_will_release(&mut self, index: usize) {
        for entry in &mut self.entries {
            entry.palette_will_release(index);
        }
    }

    fn palette_did_move(&mut self, from: usize, to: usize) {
        for entry in &mut self.entries {
            entry.palette_did_move(from, to);
        }
    }

    fn material_did_retain(&mut self, palette: usize, index: usize) {
        for entry in &mut self.entries {
            entry.material_did_retain(palette, index);
        }
    }

    fn materials_will_release(&mut self, palette: usize, indices: &[usize]) {
        for entry in &mut self.entries {
            entry.materials_will_release(palette, indices);
        }
    }

    fn materials_did_repaint(&mut self, palette: usize, remap: &[(usize, usize)]) {
        for entry in &mut self.entries {
            entry.materials_did_repaint(palette, remap);
        }
    }

    fn voxel_did_retain(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        for entry in &mut self.entries {
            entry.voxel_did_retain(object, voxel);
        }
    }

    fn voxel_will_release(&mut self, object: usize, voxel: U32Id<BVoxVoxel>) {
        for entry in &mut self.entries {
            entry.voxel_will_release(object, voxel);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        VoxMap, VoxValue,
        ext::{
            CompositeVoxExt, Error, Result, VoxExt, VoxExtEntryCodec, decode_entry, encode_entry,
        },
    };
    use std::any::Any;

    /// A format ext counting the objects retained since it loaded.
    #[derive(Clone, Debug, PartialEq)]
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

    impl VoxExt for Count {
        fn to_vox_ext(&self) -> Result<VoxMap> {
            encode_entry(self)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn clone_box(&self) -> Box<dyn VoxExt> {
            Box::new(self.clone())
        }

        fn object_did_retain(&mut self, _index: usize) {
            self.0 += 1;
        }
    }

    /// A foreign entry, then a `count` one.
    fn block() -> VoxMap {
        VoxMap(vec![
            ("other".to_owned(), VoxValue::Bool(true)),
            ("count".to_owned(), VoxValue::Number(2.0)),
        ])
    }

    /// The `count` entry decodes, the foreign one stays verbatim, and the
    /// block re-encodes in its order.
    #[test]
    fn a_block_round_trips_in_order() {
        let composite = CompositeVoxExt::from(block()).decode::<Count>().unwrap();

        assert_eq!(composite.to_vox_ext().unwrap(), block());

        let empty = CompositeVoxExt::from(VoxMap::default());

        assert_eq!(empty.to_vox_ext().unwrap(), VoxMap::default());
    }

    /// A hook reaches the decoded ext and leaves the verbatim entry alone.
    #[test]
    fn hooks_forward_to_the_decoded_exts() {
        let mut composite = CompositeVoxExt::from(block()).decode::<Count>().unwrap();

        composite.object_did_retain(0);

        let block = composite.to_vox_ext().unwrap();

        assert_eq!(decode_entry::<Count>(&block).unwrap(), Some(Count(3)));

        assert_eq!(block.0[0], ("other".to_owned(), VoxValue::Bool(true)));
    }

    /// A clone carries the decoded ext, not a verbatim copy of its entry.
    #[test]
    fn a_clone_keeps_the_decoded_exts() {
        let composite = CompositeVoxExt::from(block()).decode::<Count>().unwrap();

        let mut cloned = composite.clone_box();

        cloned.object_did_retain(0);

        assert_eq!(
            decode_entry::<Count>(&cloned.to_vox_ext().unwrap()).unwrap(),
            Some(Count(3))
        );

        assert_eq!(
            decode_entry::<Count>(&composite.to_vox_ext().unwrap()).unwrap(),
            Some(Count(2))
        );
    }

    /// An entry under a format's key that does not decode is an error.
    #[test]
    fn a_bad_entry_errors() {
        let block = VoxMap(vec![("count".to_owned(), VoxValue::Null)]);

        assert!(CompositeVoxExt::from(block).decode::<Count>().is_err());
    }

    /// A block holding a key twice errors on encode.
    #[test]
    fn a_repeated_key_errors_on_encode() {
        let block = VoxMap(vec![
            ("other".to_owned(), VoxValue::Null),
            ("other".to_owned(), VoxValue::Null),
        ]);

        assert!(CompositeVoxExt::from(block).to_vox_ext().is_err());
    }
}
