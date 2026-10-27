use branded_id::U32Id;
use ty_math::TyTransformF64;
use voxcore::BVoxObject;

/// One placement of an object: the transform from its grid units onto world
/// meters. A voxel at `p` fills the unit cube with min corner `p` before the
/// transform applies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPlacement {
    /// The placed object.
    pub object_id: U32Id<BVoxObject>,

    /// The transform.
    pub transform: TyTransformF64,
}
