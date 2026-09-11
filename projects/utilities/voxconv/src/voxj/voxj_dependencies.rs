use crate::ForwardDependencies;
use voxj::dependencies::{CostVoxjObject, DecodeBase64, EncodeBase64};
use voxj_voxcore::codec::dependencies::{DecodeVoxjJson, Deflate, EncodeVoxjJson, Inflate};

/// The Voxel Json codec's dependencies a caller supplies. The codec's
/// dependencies cover voxj's too.
pub trait VoxjDependencies {
    /// The Voxel Json codec's dependencies.
    type Voxj: DecodeBase64
        + EncodeBase64
        + CostVoxjObject
        + DecodeVoxjJson
        + EncodeVoxjJson
        + Inflate
        + Deflate;

    /// The Voxel Json codec's dependencies.
    fn voxj(&self) -> &Self::Voxj;
}

/// Forwards to the target's.
impl<T: ForwardDependencies<Target: VoxjDependencies>> VoxjDependencies for T {
    type Voxj = <T::Target as VoxjDependencies>::Voxj;

    fn voxj(&self) -> &Self::Voxj {
        self.target().voxj()
    }
}
