use crate::{
    Dependencies, Format, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    mvox::MVox,
};
use mvox_voxcore::{
    codec::{from_mvox_bytes_with_ext, to_mvox_bytes_with_ext},
    ext::MVoxExt,
};
use voxcore::VoxMapEntry;

/// The key of the MagicaVoxel slot in a Voxel Json `ext` block.
const KEY: &str = "mvox";

impl FormatExt for MVox {
    fn read_with_ext<D: Dependencies>(
        _dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain> {
        Ok(box_ext(from_mvox_bytes_with_ext(
            VoxDocumentFile::single_bytes(files)?,
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &(),
        state: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let Some(ext) = find_ext::<MVoxExt>(KEY, state.ext().as_ref())? else {
            return Self::write(dependencies, options, &state.take_ext().0);
        };

        let bytes = to_mvox_bytes_with_ext(&state.take_ext().0.put_ext(ext))?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        push_ext_slot::<MVoxExt>(KEY, ext, slots)
    }

    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        ext_from_slot::<MVoxExt>(KEY, slot)
    }
}
