use branded_id::U32Id;
use ty_math::TyVector4F64;
use voxcore::BVoxMaterial;

/// A candidate material as a clustering point. Coordinates past the reading's
/// dimension stay zero.
#[derive(Clone, Copy, Debug)]
pub struct QuantizePoint {
    /// The material the point places.
    pub(crate) material_id: U32Id<BVoxMaterial>,

    /// The material's value in the clustering space.
    pub(crate) coords: TyVector4F64,

    /// How many live voxels of the quantized layers sample the material.
    pub(crate) population: u64,
}
