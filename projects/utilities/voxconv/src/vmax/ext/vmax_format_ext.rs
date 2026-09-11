use crate::{
    Dependencies, Format, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    vmax::{VMax, VMaxWriteOptions, package_file},
};
use vmax_voxcore::{
    codec::{from_vmax_package_with_ext, to_vmax_package_with_ext},
    ext::VMaxExt,
};
use voxcore::VoxMapEntry;

/// The key of the Voxel Max slot in a Voxel Json `ext` block.
const KEY: &str = "vmax";

impl FormatExt for VMax {
    fn read_with_ext<D: Dependencies>(
        dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<VoxconvVoxMain> {
        let paths = files.iter().map(|file| file.path.clone()).collect();

        Ok(box_ext(from_vmax_package_with_ext(
            dependencies.vmax(),
            || Ok(paths),
            |path| Ok(package_file(files, path)),
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &VMaxWriteOptions,
        state: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let Some(ext) = find_ext::<VMaxExt>(KEY, state.ext().as_ref())? else {
            return Self::write(dependencies, options, &state.take_ext().0);
        };

        let state = state.take_ext().0.put_ext(ext);

        let mut files = Vec::new();

        to_vmax_package_with_ext(dependencies.vmax(), &state, options, |path, bytes| {
            files.push(VoxDocumentFile::new(path, bytes.to_vec()));

            Ok(())
        })?;

        Ok(files)
    }

    fn encode_slots(ext: &dyn VoxconvExt, slots: &mut Vec<VoxMapEntry>) -> Result<bool> {
        push_ext_slot::<VMaxExt>(KEY, ext, slots)
    }

    fn decode_slot(slot: &VoxMapEntry) -> Result<Option<Box<dyn VoxconvExt>>> {
        ext_from_slot::<VMaxExt>(KEY, slot)
    }
}
