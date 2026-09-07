use crate::{Result, VoxjFileBuilder};
use voxcore::ext::VoxExt;
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::{
    Deflate, EncodeVoxjJson, to_voxj_file_bytes, to_voxj_pretty_file_bytes, to_voxjz_file_bytes,
};

/// Serializes a [`VoxjFileBuilder`]'s document to bytes, one method per
/// container form.
pub trait VoxjFileBuilderCodec {
    /// Builds the document and serializes it to compact `.voxj` JSON bytes.
    fn to_voxj_bytes(self) -> Result<Vec<u8>>;

    /// Builds the document and serializes it to pretty-printed `.voxj` JSON
    /// bytes.
    fn to_voxj_pretty_bytes(self) -> Result<Vec<u8>>;

    /// Builds the document and serializes it to a `.voxjz` zip archive
    /// holding one compact `.voxj` member.
    fn to_voxjz_bytes(self) -> Result<Vec<u8>>;
}

impl<T: VoxExt, D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson + Deflate> VoxjFileBuilderCodec
    for VoxjFileBuilder<'_, T, D>
{
    fn to_voxj_bytes(self) -> Result<Vec<u8>> {
        let dependencies = self.dependencies;

        Ok(to_voxj_file_bytes(dependencies, &self.build()?))
    }

    fn to_voxj_pretty_bytes(self) -> Result<Vec<u8>> {
        let dependencies = self.dependencies;

        Ok(to_voxj_pretty_file_bytes(dependencies, &self.build()?))
    }

    fn to_voxjz_bytes(self) -> Result<Vec<u8>> {
        let dependencies = self.dependencies;

        Ok(to_voxjz_file_bytes(dependencies, &self.build()?))
    }
}
