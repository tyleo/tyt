/// The axes a 2D mirror reflects.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SdfAxes2d {
    /// The u axis.
    U,

    /// The u and v axes.
    Uv,

    /// The v axis.
    V,
}
