use crate::{Dependencies, Format, ReadFormat, Result, VoxDocumentFile, WriteFormat};
use mvox_voxcore::{
    codec::{from_mvox_bytes, to_mvox_bytes},
    to_mvox_vox_main,
};
use voxcore::VoxMain;

/// MagicaVoxel, the `.vox` file. The codec needs no dependencies.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct MVox;

impl Format for MVox {
    type WriteOptions = ();

    const NAME: &'static str = "mvox";

    const EXTENSIONS: &'static [&'static str] = &["vox"];

    fn read_format() -> ReadFormat {
        ReadFormat::MVox
    }

    fn write_format(_options: ()) -> WriteFormat {
        WriteFormat::MVox
    }

    fn read<D: Dependencies>(_dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
        Ok(from_mvox_bytes(VoxDocumentFile::single_bytes(files)?)?
            .take_ext()
            .state)
    }

    fn write<D: Dependencies>(
        _dependencies: &D,
        _options: &(),
        state: VoxMain<()>,
    ) -> Result<Vec<VoxDocumentFile>> {
        let bytes = to_mvox_bytes(&to_mvox_vox_main(state)?)?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }
}
