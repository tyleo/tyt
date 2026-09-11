use crate::{
    Dependencies, Format, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    qbcl::Qbt,
};
use qbcl_voxcore::{
    codec::{from_qbt_bytes_with_ext, to_qbt_bytes_with_ext},
    ext::QbtExt,
};
use voxcore::VoxMapEntry;

/// The key of the Qubicle Binary Tree slot in a Voxel Json `ext` block.
const KEY: &str = "qbt";

impl FormatExt for Qbt {
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain> {
        Ok(box_ext(from_qbt_bytes_with_ext(
            dependencies.qbcl(),
            VoxDocumentFile::single_bytes(files)?,
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &(),
        state: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let Some(ext) = find_ext::<QbtExt>(KEY, state.ext().as_ref())? else {
            return Self::write(dependencies, options, &state.take_ext().0);
        };

        let bytes = to_qbt_bytes_with_ext(dependencies.qbcl(), &state.take_ext().0.put_ext(ext))?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        push_ext_slot::<QbtExt>(KEY, ext, slots)
    }

    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        ext_from_slot::<QbtExt>(KEY, slot)
    }
}
