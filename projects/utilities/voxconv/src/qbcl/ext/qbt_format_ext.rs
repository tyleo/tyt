use crate::{
    Dependencies, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    qbcl::Qbt,
};
use qbcl_voxcore::{
    QbtExt,
    codec::{from_qbt_bytes, to_qbt_bytes},
    to_qbt_vox_main,
};
use voxcore::VoxMapEntry;

/// The key of the Qubicle Binary Tree slot in a Voxel Json `ext` block.
const KEY: &str = "qbt";

impl FormatExt for Qbt {
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain> {
        Ok(box_ext(from_qbt_bytes(
            dependencies.qbcl(),
            VoxDocumentFile::single_bytes(files)?,
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        _options: &(),
        state: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let state = match find_ext::<QbtExt>(KEY, state.ext().as_ref())? {
            Some(ext) => state.take_ext().state.put_ext(ext),
            None => to_qbt_vox_main(state.take_ext().state)?,
        };

        let bytes = to_qbt_bytes(dependencies.qbcl(), &state)?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        push_ext_slot::<QbtExt>(KEY, ext, slots)
    }

    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        ext_from_slot::<QbtExt>(KEY, slot)
    }
}
