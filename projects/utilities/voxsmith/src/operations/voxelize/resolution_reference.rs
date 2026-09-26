/// The side a resolution count divides. World references measure every
/// placed object's bounds together, after every node transform. Object
/// references measure each object's bounds and take the extreme across
/// objects. A side with no extent never counts as shortest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionReference {
    /// The longest side of the world bounds.
    LongestWorld,

    /// The shortest side of the world bounds.
    ShortestWorld,

    /// The world bounds' extent along x.
    WorldX,

    /// The world bounds' extent along y.
    WorldY,

    /// The world bounds' extent along z.
    WorldZ,

    /// The longest side of any object's bounds.
    LongestObject,

    /// The shortest side of any object's bounds.
    ShortestObject,

    /// The longest extent along x of any object's bounds.
    LongestObjectX,

    /// The longest extent along y of any object's bounds.
    LongestObjectY,

    /// The longest extent along z of any object's bounds.
    LongestObjectZ,

    /// The shortest extent along x of any object's bounds.
    ShortestObjectX,

    /// The shortest extent along y of any object's bounds.
    ShortestObjectY,

    /// The shortest extent along z of any object's bounds.
    ShortestObjectZ,
}
