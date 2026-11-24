use crate::BTileObject;
use branded_id::UsizeId;
use ty_math::TyTransformF64;

/// One placement of a [`TileScene`](crate::TileScene)'s object.
#[derive(Clone, Copy, Debug)]
pub struct TilePlacement {
    pub object: UsizeId<BTileObject>,

    /// What carries the object's cells to the world.
    pub transform: TyTransformF64,
}
