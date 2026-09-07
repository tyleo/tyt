use crate::{Result, VmaxFileBuilder};
use vmax_codec::{
    CompressLzfse, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson, Result as CodecResult,
    to_vmax_package as write_vmax_package,
};

/// Emits a [`VmaxFileBuilder`]'s document as a package's files.
pub trait VmaxFileBuilderCodec {
    /// Builds the document and writes each file through `write` with its
    /// package-relative path and bytes.
    fn to_vmax_package<D, W>(self, dependencies: &D, write: W) -> Result<()>
    where
        D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
        W: FnMut(&str, &[u8]) -> CodecResult<()>;
}

impl<T> VmaxFileBuilderCodec for VmaxFileBuilder<'_, T> {
    fn to_vmax_package<D, W>(self, dependencies: &D, write: W) -> Result<()>
    where
        D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
        W: FnMut(&str, &[u8]) -> CodecResult<()>,
    {
        let file = self.build()?;

        Ok(write_vmax_package(dependencies, &file, write)?)
    }
}
