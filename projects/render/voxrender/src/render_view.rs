use crate::RenderProjection;
use ty_math::TyPoseF64;

/// A camera in world space, looking down its pose's -Z with +Y up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderView {
    /// The pose.
    pub pose: TyPoseF64,

    /// The projection.
    pub projection: RenderProjection,
}
