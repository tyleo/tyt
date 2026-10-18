use branded_id::U32Id;
use voxcore::BVoxVoxel;
use voxrender::BRenderPlacement;
use voxsurface::SurfaceSpan;

/// One surface hit of a ray: the face through which it entered a live cell
/// of a material other than the one it was inside, or left one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderHit {
    /// The placement the ray hit.
    pub placement_id: U32Id<BRenderPlacement>,

    /// The cell the ray hit.
    pub voxel_id: U32Id<BVoxVoxel>,

    /// The face of the hit cell the ray crossed, a unit span in the object's
    /// grid units. An entry's normal faces the ray and an exit's faces away
    /// from it.
    pub face: SurfaceSpan,

    /// How far along the face's `u` and `v` the hit landed, each in
    /// `[0, 1]`.
    pub along: [f64; 2],

    /// The hit point in the object's grid, rounded to fixed point. It lies
    /// on the face's plane.
    pub point: [i32; 3],

    /// The distance from the ray's origin, in meters.
    pub distance: f64,

    /// Whether the ray left the cell through the face.
    pub exit: bool,
}
