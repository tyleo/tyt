/// The axes a 3D mirror reflects.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SdfAxes3d {
    /// The x axis.
    X,

    /// The x and y axes.
    Xy,

    /// The x, y, and z axes.
    Xyz,

    /// The x and z axes.
    Xz,

    /// The y axis.
    Y,

    /// The y and z axes.
    Yz,

    /// The z axis.
    Z,
}
