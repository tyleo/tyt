use crate::{
    Dependencies, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    goxl::Goxl,
};
use goxl_voxcore::{
    GoxlExt,
    codec::{from_goxl_bytes, to_goxl_bytes},
    to_goxl_vox_main,
};
use voxcore::VoxMapEntry;

/// The key of the Goxel slot in a Voxel Json `ext` block.
const KEY: &str = "goxl";

impl FormatExt for Goxl {
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain> {
        Ok(box_ext(from_goxl_bytes(
            dependencies.goxl(),
            VoxDocumentFile::single_bytes(files)?,
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        _options: &(),
        main: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let main = match find_ext::<GoxlExt>(KEY, main.ext().as_ref())? {
            Some(ext) => main.take_ext().main.put_ext(ext),
            None => to_goxl_vox_main(main.take_ext().main)?,
        };

        let bytes = to_goxl_bytes(dependencies.goxl(), &main)?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        push_ext_slot::<GoxlExt>(KEY, ext, slots)
    }

    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        ext_from_slot::<GoxlExt>(KEY, slot)
    }
}
