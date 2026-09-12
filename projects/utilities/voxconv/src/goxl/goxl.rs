use crate::{Dependencies, Format, ReadFormat, Result, VoxDocumentFile, WriteFormat};
use goxl_voxcore::{
    codec::{from_goxl_bytes, to_goxl_bytes},
    to_goxl_vox_main,
};
use voxcore::VoxMain;

/// Goxel, the `.gox` file.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Goxl;

impl Format for Goxl {
    type WriteOptions = ();

    const NAME: &'static str = "goxl";

    const EXTENSIONS: &'static [&'static str] = &["gox"];

    fn read_format() -> ReadFormat {
        ReadFormat::Goxl
    }

    fn write_format(_options: ()) -> WriteFormat {
        WriteFormat::Goxl
    }

    fn read<D: Dependencies>(dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
        Ok(
            from_goxl_bytes(dependencies.goxl(), VoxDocumentFile::single_bytes(files)?)?
                .take_ext()
                .main,
        )
    }

    fn write<D: Dependencies>(
        dependencies: &D,
        _options: &(),
        main: VoxMain<()>,
    ) -> Result<Vec<VoxDocumentFile>> {
        let bytes = to_goxl_bytes(dependencies.goxl(), &to_goxl_vox_main(main)?)?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }
}
