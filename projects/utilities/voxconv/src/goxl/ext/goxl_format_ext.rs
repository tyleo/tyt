use crate::{
    Dependencies, Format, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    goxl::Goxl,
};
use goxl_voxcore::{
    codec::{from_goxl_bytes_with_ext, to_goxl_bytes_with_ext},
    ext::GoxlExt,
};
use voxcore::VoxMapEntry;

/// The key of the Goxel slot in a Voxel Json `ext` block.
const KEY: &str = "goxl";

impl FormatExt for Goxl {
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain> {
        Ok(box_ext(from_goxl_bytes_with_ext(
            dependencies.goxl(),
            VoxDocumentFile::single_bytes(files)?,
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &(),
        state: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let Some(ext) = find_ext::<GoxlExt>(KEY, state.ext().as_ref())? else {
            return Self::write(dependencies, options, &state.take_ext().0);
        };

        let bytes = to_goxl_bytes_with_ext(dependencies.goxl(), &state.take_ext().0.put_ext(ext))?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        push_ext_slot::<GoxlExt>(KEY, ext, slots)
    }

    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        ext_from_slot::<GoxlExt>(KEY, slot)
    }
}
