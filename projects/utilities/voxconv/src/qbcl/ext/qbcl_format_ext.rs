use crate::{
    Dependencies, Format, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    qbcl::Qbcl,
};
use qbcl_voxcore::{
    codec::{from_qbcl_bytes_with_ext, to_qbcl_bytes_with_ext},
    ext::QbclExt,
};
use voxcore::VoxMapEntry;

/// The key of the Qubicle Construction Library slot in a Voxel Json `ext`
/// block.
const KEY: &str = "qbcl";

impl FormatExt for Qbcl {
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain> {
        Ok(box_ext(from_qbcl_bytes_with_ext(
            dependencies.qbcl(),
            VoxDocumentFile::single_bytes(files)?,
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &(),
        state: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let Some(ext) = find_ext::<QbclExt>(KEY, state.ext().as_ref())? else {
            return Self::write(dependencies, options, &state.take_ext().0);
        };

        let bytes = to_qbcl_bytes_with_ext(dependencies.qbcl(), &state.take_ext().0.put_ext(ext))?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        push_ext_slot::<QbclExt>(KEY, ext, slots)
    }

    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        ext_from_slot::<QbclExt>(KEY, slot)
    }
}
