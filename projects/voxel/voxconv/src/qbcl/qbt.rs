use crate::{Dependencies, Format, ReadFormat, Result, VoxDocumentFile, WriteFormat};
use qbcl_voxcore::{
    codec::{from_qbt_bytes, to_qbt_bytes},
    to_qbt_vox_main,
};
use voxcore::VoxMain;

/// Qubicle Binary Tree, the `.qbt` file.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Qbt;

impl Format for Qbt {
    type WriteOptions = ();

    const NAME: &'static str = "qbt";

    const EXTENSIONS: &'static [&'static str] = &["qbt"];

    fn read_format() -> ReadFormat {
        ReadFormat::Qbt
    }

    fn write_format(_options: ()) -> WriteFormat {
        WriteFormat::Qbt
    }

    fn read<D: Dependencies>(dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
        Ok(
            from_qbt_bytes(dependencies.qbcl(), VoxDocumentFile::single_bytes(files)?)?
                .take_ext()
                .main,
        )
    }

    fn write<D: Dependencies>(
        dependencies: &D,
        _options: &(),
        main: VoxMain<()>,
    ) -> Result<Vec<VoxDocumentFile>> {
        let bytes = to_qbt_bytes(dependencies.qbcl(), &to_qbt_vox_main(main)?)?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }
}
