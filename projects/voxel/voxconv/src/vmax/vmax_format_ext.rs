use crate::{
    Dependencies, Result, VoxDocumentFile,
    ext::{FormatExt, VoxconvExt, VoxconvVoxMain, box_ext, ext_from_slot, find_ext, push_ext_slot},
    vmax::{VMax, VMaxWriteOptions, package_file},
};
use vmax_voxcore::{
    VMaxExt,
    codec::{from_vmax_package, to_vmax_package},
    to_vmax_vox_main,
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

        Ok(box_ext(from_vmax_package(
            dependencies.vmax(),
            || Ok(paths),
            |path| Ok(package_file(files, path)),
        )?))
    }

    fn write_with_ext<D: Dependencies>(
        dependencies: &D,
        options: &VMaxWriteOptions,
        main: VoxconvVoxMain,
    ) -> Result<Vec<VoxDocumentFile>> {
        let main = match find_ext::<VMaxExt>(KEY, main.ext().as_ref())? {
            Some(ext) => main.take_ext().main.put_ext(ext),
            None => to_vmax_vox_main(main.take_ext().main)?,
        };

        let mut files = Vec::new();

        to_vmax_package(dependencies.vmax(), &main, options, |path, bytes| {
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
