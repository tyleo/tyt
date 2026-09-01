use crate::{EditStateMode, PositionEncoding, Result, SampleEncoding, VOXJ_DEPENDENCIES};
use voxcore::{VoxMain, ext::VoxExtSlot};
use voxj::{DependenciesImpl as VoxjDependenciesImpl, VoxjFile};
use voxj_codec::{to_voxj_file_bytes, to_voxj_pretty_file_bytes, to_voxjz_file_bytes};
use voxj_voxcore::VoxjFileBuilder as RawVoxjFileBuilder;

/// Builds a Voxel Json document from a [`VoxMain`], the configurable form of
/// [`to_voxj_file`](crate::to_voxj_file), [`to_voxj_bytes`](crate::to_voxj_bytes),
/// and [`to_voxjz_bytes`](crate::to_voxjz_bytes). It defaults to the smallest
/// per-object block encodings, persists the slot's `ext` block, and records
/// the edit state automatically, reproducing the document those functions
/// write. Each byte terminal serializes the built document in one container
/// form.
pub struct VoxjFileBuilder<'a, T>(RawVoxjFileBuilder<'a, T, VoxjDependenciesImpl>);

impl<'a, T: VoxExtSlot> VoxjFileBuilder<'a, T> {
    /// Starts a builder encoding `state` into a Voxel Json document.
    pub fn new(state: &'a VoxMain<T>) -> Self {
        Self(RawVoxjFileBuilder::new(&VOXJ_DEPENDENCIES, state))
    }

    /// Sets the position-block encoding, or `None` to search for the smallest
    /// paired with the sample encoding.
    pub fn position_encoding(self, position_encoding: Option<PositionEncoding>) -> Self {
        Self(self.0.position_encoding(position_encoding))
    }

    /// Sets the sample-block encoding, or `None` to search for the smallest
    /// paired with the position encoding.
    pub fn sample_encoding(self, sample_encoding: Option<SampleEncoding>) -> Self {
        Self(self.0.sample_encoding(sample_encoding))
    }

    /// Keeps (the default) or drops the slot's `ext` extension block.
    pub fn ext(self, ext: bool) -> Self {
        Self(self.0.ext(ext))
    }

    /// Sets when each object's editor build volume is recorded in the
    /// document's edit state.
    pub fn edit_state(self, edit_state: EditStateMode) -> Self {
        Self(self.0.edit_state(edit_state))
    }

    /// Builds the [`VoxjFile`].
    pub fn build(self) -> Result<VoxjFile> {
        Ok(self.0.build()?)
    }

    /// Builds the document and serializes it to compact `.voxj` JSON bytes.
    pub fn to_voxj_bytes(self) -> Result<Vec<u8>> {
        Ok(to_voxj_file_bytes(&self.build()?)?)
    }

    /// Builds the document and serializes it to pretty-printed `.voxj` JSON
    /// bytes.
    pub fn to_voxj_pretty_bytes(self) -> Result<Vec<u8>> {
        Ok(to_voxj_pretty_file_bytes(&self.build()?)?)
    }

    /// Builds the document and serializes it to a `.voxjz` zip archive
    /// holding one compact `.voxj` member.
    pub fn to_voxjz_bytes(self) -> Result<Vec<u8>> {
        Ok(to_voxjz_file_bytes(&self.build()?)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        EditStateMode, PositionEncoding, SampleEncoding, VoxjCheckStatus, VoxjFileBuilder,
        VoxjVoxMain, check_voxj_bytes, from_voxj_bytes, to_voxj_bytes, to_voxj_vox_main,
        to_voxjz_bytes, voxj_version_from_bytes,
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxMap, VoxObject, VoxPalette, VoxValue, VoxValuePool};
    use voxj_codec::from_voxj_file_bytes;

    /// One `baseColor` palette and one tight 1x1x1 object sampling its one
    /// material, carrying `ext` in the slot.
    fn state<T>(ext: T) -> VoxMain<T> {
        let mut state = VoxMain::default();

        let colors_value_pool_id =
            state.retain_value_pool(VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());

        let mut palette = VoxPalette::default();

        palette
            .retain_property(
                "baseColor".to_owned(),
                colors_value_pool_id,
                U32Id::from_u32(0),
            )
            .unwrap();

        let material_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

        let palette_id = state.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();

        object.retain_layer(palette_id, material_id);

        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();

        object.retain_voxel(voxel_id, &[material_id]).unwrap();

        state.retain_object(object).unwrap();

        state.map_ext(|()| ext)
    }

    /// A block some other format owns, kept verbatim through the raw slot.
    fn block() -> VoxMap {
        VoxMap(vec![("vendor".to_owned(), VoxValue::Number(7.0))])
    }

    /// The slot rides the `ext` block through a document and back, in both
    /// containers.
    #[test]
    fn round_trips_the_slot_through_both_containers() {
        let voxj = to_voxj_bytes(&state(Some(block()))).unwrap();

        let voxjz = to_voxjz_bytes(&state(Some(block()))).unwrap();

        assert!(voxj.starts_with(b"{"));

        assert!(voxjz.starts_with(b"PK"));

        for bytes in [voxj, voxjz] {
            let loaded: VoxjVoxMain = from_voxj_bytes(&bytes).unwrap();

            assert_eq!(loaded.ext(), &Some(block()));

            assert_eq!(loaded.object_count(), 1);
        }
    }

    /// The unit slot and a builder dropping the block both write a document
    /// whose block is absent.
    #[test]
    fn declining_the_block_writes_none() {
        let unit = to_voxj_bytes(&state(())).unwrap();

        let dropped = VoxjFileBuilder::new(&state(Some(block())))
            .ext(false)
            .to_voxj_bytes()
            .unwrap();

        for bytes in [unit, dropped] {
            let loaded: VoxjVoxMain = from_voxj_bytes(&bytes).unwrap();

            assert_eq!(loaded.ext(), &None);
        }
    }

    /// The block form carries a typed slot's block verbatim.
    #[test]
    fn to_voxj_vox_main_encodes_the_slot() {
        let raw = to_voxj_vox_main(state(Some(block()))).unwrap();

        assert_eq!(raw.ext(), &Some(block()));

        assert_eq!(to_voxj_vox_main(state(())).unwrap().ext(), &None);
    }

    /// The builder's encodings and edit state reach the document, and its
    /// pretty form parses to the same document as the compact one.
    #[test]
    fn builder_settings_reach_the_document() {
        let state = state(());

        let builder = || {
            VoxjFileBuilder::new(&state)
                .position_encoding(Some(PositionEncoding::RawJson))
                .sample_encoding(Some(SampleEncoding::RawJson))
                .edit_state(EditStateMode::Always)
        };

        let compact = builder().build().unwrap();

        let pretty = builder().to_voxj_pretty_bytes().unwrap();

        assert!(compact.main.edit_state.is_some());

        assert!(pretty.starts_with(b"{\n"));

        assert_eq!(from_voxj_file_bytes(&pretty).unwrap(), compact);
    }

    /// A written document passes every check and reports the version it was
    /// stamped with.
    #[test]
    fn a_written_document_checks_clean() {
        let bytes = to_voxj_bytes(&state(())).unwrap();

        assert_eq!(voxj_version_from_bytes(&bytes).unwrap(), 1);

        let checks = check_voxj_bytes(&bytes).unwrap();

        assert!(!checks.is_empty());

        assert!(checks.iter().all(|check| {
            matches!(
                check.status,
                VoxjCheckStatus::Passed | VoxjCheckStatus::Unverifiable
            )
        }));
    }

    /// Undecodable bytes are an error.
    #[test]
    fn undecodable_bytes_error() {
        assert!(check_voxj_bytes(b"not a document").is_err());

        assert!(voxj_version_from_bytes(b"not a document").is_err());
    }
}
