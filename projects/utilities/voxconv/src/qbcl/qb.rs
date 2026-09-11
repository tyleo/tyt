use crate::{Dependencies, Format, ReadFormat, Result, VoxDocumentFile, WriteFormat};
use qbcl_voxcore::codec::{from_qb_bytes, to_qb_bytes};
use voxcore::VoxMain;

/// Qubicle Binary, the `.qb` file.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Qb;

impl Format for Qb {
    type WriteOptions = ();

    const NAME: &'static str = "qb";

    const EXTENSIONS: &'static [&'static str] = &["qb"];

    fn read_format() -> ReadFormat {
        ReadFormat::Qb
    }

    fn write_format(_options: ()) -> WriteFormat {
        WriteFormat::Qb
    }

    fn read<D: Dependencies>(_dependencies: &D, files: &[VoxDocumentFile]) -> Result<VoxMain<()>> {
        Ok(from_qb_bytes(VoxDocumentFile::single_bytes(files)?)?)
    }

    fn write<D: Dependencies>(
        _dependencies: &D,
        _options: &(),
        state: &VoxMain<()>,
    ) -> Result<Vec<VoxDocumentFile>> {
        let bytes = to_qb_bytes(state)?;

        Ok(vec![VoxDocumentFile::single(bytes)])
    }
}
