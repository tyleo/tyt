use ty_math::{TyMatrix4x4F64, TyVector3F64};

/// One eye of a [`BoxView`](crate::BoxView)'s frame.
#[derive(Clone, Copy, Debug)]
pub struct ViewEye {
    /// The matrix that carries a world point to the eye's clip space. Depth
    /// runs reversed.
    pub world_to_clip: TyMatrix4x4F64,

    /// The eye in the world.
    pub position: TyVector3F64,
}
