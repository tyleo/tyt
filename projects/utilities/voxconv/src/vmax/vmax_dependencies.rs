use crate::ForwardDependencies;
use vmax_voxcore::codec::dependencies::{
    CompressLzfse, DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse, EncodePng,
    EncodeVMaxPlist, EncodeVMaxSceneJson,
};

/// The Voxel Max codec's dependencies a caller supplies.
pub trait VMaxDependencies {
    /// The Voxel Max codec's dependencies.
    type VMax: CompressLzfse
        + DecompressLzfse
        + DecodeVMaxPlist
        + EncodeVMaxPlist
        + DecodePng
        + EncodePng
        + DecodeVMaxSceneJson
        + EncodeVMaxSceneJson;

    /// The Voxel Max codec's dependencies.
    fn vmax(&self) -> &Self::VMax;
}

/// Forwards to the target's.
impl<T: ForwardDependencies<Target: VMaxDependencies>> VMaxDependencies for T {
    type VMax = <T::Target as VMaxDependencies>::VMax;

    fn vmax(&self) -> &Self::VMax {
        self.target().vmax()
    }
}
