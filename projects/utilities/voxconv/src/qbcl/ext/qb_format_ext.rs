use crate::{
    Dependencies, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    qbcl::Qb,
};
use qbcl_voxcore::{
    QbExt,
    codec::{from_qb_bytes, to_qb_bytes},
    to_qb_vox_main,
};
use voxcore::VoxMapEntry;

/// The key of the Qubicle Binary slot in a Voxel Json `ext` block.
const KEY: &str = "qb";

impl FormatExt for Qb {
    fn read_with_ext<D: Dependencies>(
        _dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain> {
        Ok(box_ext(from_qb_bytes(VoxDocumentFile::single_bytes(
            files,
        )?)?))
    }

    fn write_with_ext<D: Dependencies>(
        _dependencies: &D,
        _options: &(),
        state: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let state = match find_ext::<QbExt>(KEY, state.ext().as_ref())? {
            Some(ext) => state.take_ext().state.put_ext(ext),
            None => to_qb_vox_main(state.take_ext().state)?,
        };

        let bytes = to_qb_bytes(&state)?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        push_ext_slot::<QbExt>(KEY, ext, slots)
    }

    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        ext_from_slot::<QbExt>(KEY, slot)
    }
}
