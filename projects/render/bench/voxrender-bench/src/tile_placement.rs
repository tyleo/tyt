use ty_math::TyTransformF64;

/// One placement of a [`TileScene`](crate::TileScene)'s object.
#[derive(Clone, Copy, Debug)]
pub struct TilePlacement {
    /// The object's place among the scene's.
    pub object: usize,

    /// What carries the object's cells to the world.
    pub transform: TyTransformF64,
}
