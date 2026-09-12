use crate::{
    Dependencies, Format, ReadFormat, Result, VoxDocumentFile, WriteFormat,
    vmax::{VMaxWriteOptions, package_file},
};
use vmax_voxcore::{
    codec::{from_vmax_package, to_vmax_package},
    to_vmax_vox_main,
};
use voxcore::VoxMain;

/// Voxel Max, the `.vmax` package directory.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct VMax;

impl Format for VMax {
    type WriteOptions = VMaxWriteOptions;

    const NAME: &'static str = "vmax";

    const EXTENSIONS: &'static [&'static str] = &["vmax"];

    const PACKAGE: bool = true;

    fn read_format() -> ReadFormat {
        ReadFormat::VMax
    }

    fn write_format(options: VMaxWriteOptions) -> WriteFormat {
        WriteFormat::VMax(options)
    }

    fn read<D: Dependencies>(dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
        let paths = files.iter().map(|file| file.path.clone()).collect();

        Ok(from_vmax_package(
            dependencies.vmax(),
            || Ok(paths),
            |path| Ok(package_file(files, path)),
        )?
        .take_ext()
        .state)
    }

    fn write<D: Dependencies>(
        dependencies: &D,
        options: &VMaxWriteOptions,
        state: VoxMain<()>,
    ) -> Result<Vec<VoxDocumentFile>> {
        let mut files = Vec::new();

        let state = to_vmax_vox_main(state)?;

        to_vmax_package(dependencies.vmax(), &state, options, |path, bytes| {
            files.push(VoxDocumentFile::new(path, bytes.to_vec()));

            Ok(())
        })?;

        Ok(files)
    }
}
