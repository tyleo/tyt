use crate::{Dependencies, Format, ReadFormat, Result, VoxDocumentFile, WriteFormat};
use qbcl_voxcore::{
    codec::{from_qbcl_bytes, to_qbcl_bytes},
    to_qbcl_vox_main,
};
use voxcore::VoxMain;

/// Qubicle Construction Library, the `.qbcl` file.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Qbcl;

impl Format for Qbcl {
    type WriteOptions = ();

    const NAME: &'static str = "qbcl";

    const EXTENSIONS: &'static [&'static str] = &["qbcl"];

    fn read_format() -> ReadFormat {
        ReadFormat::Qbcl
    }

    fn write_format(_options: ()) -> WriteFormat {
        WriteFormat::Qbcl
    }

    fn read<D: Dependencies>(dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
        Ok(
            from_qbcl_bytes(dependencies.qbcl(), VoxDocumentFile::single_bytes(files)?)?
                .take_ext()
                .main,
        )
    }

    fn write<D: Dependencies>(
        dependencies: &D,
        _options: &(),
        main: VoxMain<()>,
    ) -> Result<Vec<VoxDocumentFile>> {
        let bytes = to_qbcl_bytes(dependencies.qbcl(), &to_qbcl_vox_main(main)?)?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }
}
