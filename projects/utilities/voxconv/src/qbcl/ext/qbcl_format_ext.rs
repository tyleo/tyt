use crate::{
    Dependencies, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    qbcl::Qbcl,
};
use qbcl_voxcore::{
    QbclExt,
    codec::{from_qbcl_bytes, to_qbcl_bytes},
    to_qbcl_vox_main,
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
        Ok(box_ext(from_qbcl_bytes(
            dependencies.qbcl(),
            VoxDocumentFile::single_bytes(files)?,
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        _options: &(),
        main: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let main = match find_ext::<QbclExt>(KEY, main.ext().as_ref())? {
            Some(ext) => main.take_ext().main.put_ext(ext),
            None => to_qbcl_vox_main(main.take_ext().main)?,
        };

        let bytes = to_qbcl_bytes(dependencies.qbcl(), &main)?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        push_ext_slot::<QbclExt>(KEY, ext, slots)
    }

    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        ext_from_slot::<QbclExt>(KEY, slot)
    }
}
