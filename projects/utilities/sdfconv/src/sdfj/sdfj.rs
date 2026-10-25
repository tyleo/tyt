use crate::{Dependencies, Format, ReadFormat, Result, WriteFormat, sdfj::SdfjSerialization};
use sdfcore::SdfMain;
use sdfj_sdfcore::codec::{
    from_sdfj_bytes, from_sdfj_bytes_with_fonts, to_sdfj_bytes, to_sdfj_pretty_bytes,
    to_sdfjz_bytes,
};
use std::path::Path;

/// SDF Json, the `.sdfj` and `.sdfjz` documents.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Sdfj;

impl Format for Sdfj {
    type WriteOptions = SdfjSerialization;

    const NAME: &'static str = "sdfj";

    const EXTENSIONS: &'static [&'static str] = &["sdfj", "sdfjz"];

    fn read_format() -> ReadFormat {
        ReadFormat::Sdfj
    }

    fn write_format(options: SdfjSerialization) -> WriteFormat {
        WriteFormat::Sdfj(options)
    }

    fn extension(options: &SdfjSerialization) -> &'static str {
        options.extension()
    }

    fn read<D: Dependencies>(dependencies: &D, bytes: &[u8]) -> Result<SdfMain> {
        Ok(from_sdfj_bytes(dependencies.sdfj(), bytes)?)
    }

    fn read_with_fonts<D: Dependencies>(
        dependencies: &D,
        bytes: &[u8],
        directory: &Path,
    ) -> Result<SdfMain> {
        Ok(from_sdfj_bytes_with_fonts(
            dependencies.sdfj(),
            bytes,
            directory,
        )?)
    }

    fn write<D: Dependencies>(
        dependencies: &D,
        options: &SdfjSerialization,
        main: &SdfMain,
    ) -> Result<Vec<u8>> {
        Ok(match options {
            SdfjSerialization::Compact => to_sdfj_bytes(dependencies.sdfj(), main),
            SdfjSerialization::Pretty => to_sdfj_pretty_bytes(dependencies.sdfj(), main),
            SdfjSerialization::Zip => to_sdfjz_bytes(dependencies.sdfj(), main),
        })
    }
}
