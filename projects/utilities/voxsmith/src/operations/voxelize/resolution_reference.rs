/// The side a resolution count divides. World references measure the mesh's
/// bounds after every node transform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionReference {
    /// The longest side of the world bounds.
    LongestWorld,

    /// The shortest positive side of the world bounds.
    ShortestWorld,

    /// The world bounds' extent along x.
    WorldX,

    /// The world bounds' extent along y.
    WorldY,

    /// The world bounds' extent along z.
    WorldZ,
}
