use crate::{
    Dependencies, Format, ReadFormat, Result, VoxDocumentFile, WriteFormat,
    voxj::{VoxjSerialization, VoxjWriteFormat, check_from_voxj},
};
use voxcore::{VoxMain, check::VoxCheck};
use voxj_voxcore::{
    codec::{
        check_voxj_bytes, from_voxj_bytes, to_voxj_bytes, to_voxj_pretty_bytes, to_voxjz_bytes,
    },
    to_voxj_vox_main,
};

/// Voxel Json, the `.voxj` and `.voxjz` documents.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Voxj;

impl Format for Voxj {
    type WriteOptions = VoxjWriteFormat;

    const NAME: &'static str = "voxj";

    const EXTENSIONS: &'static [&'static str] = &["voxj", "voxjz"];

    fn read_format() -> ReadFormat {
        ReadFormat::Voxj
    }

    fn write_format(options: VoxjWriteFormat) -> WriteFormat {
        WriteFormat::Voxj(options)
    }

    fn extension(options: &VoxjWriteFormat) -> &'static str {
        options.serialization.extension()
    }

    /// The `ext` block drops.
    fn read<D: Dependencies>(dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
        Ok(
            from_voxj_bytes(dependencies.voxj(), VoxDocumentFile::single_bytes(files)?)?
                .take_ext()
                .main,
        )
    }

    /// The document gets no `ext` block.
    fn write<D: Dependencies>(
        dependencies: &D,
        options: &VoxjWriteFormat,
        main: VoxMain<()>,
    ) -> Result<Vec<VoxDocumentFile>> {
        let dependencies = dependencies.voxj();

        let main = to_voxj_vox_main(main);

        let bytes = match options.serialization {
            VoxjSerialization::Compact => to_voxj_bytes(dependencies, &main, &options.options)?,
            VoxjSerialization::Pretty => {
                to_voxj_pretty_bytes(dependencies, &main, &options.options)?
            }
            VoxjSerialization::Zip => to_voxjz_bytes(dependencies, &main, &options.options)?,
        };

        Ok(vec![VoxDocumentFile::single(bytes)])
    }

    /// Every Voxel Json spec check, each in voxcore's form.
    fn check<D: Dependencies>(
        dependencies: &D,
        files: &[VoxDocumentFile],
    ) -> Result<Vec<VoxCheck>> {
        Ok(
            check_voxj_bytes(dependencies.voxj(), VoxDocumentFile::single_bytes(files)?)?
                .into_iter()
                .map(check_from_voxj)
                .collect(),
        )
    }
}
