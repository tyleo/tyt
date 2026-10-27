/// One direction along an axis.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SdfSide {
    /// Toward -x.
    NegativeX,

    /// Toward -y.
    NegativeY,

    /// Toward -z.
    NegativeZ,

    /// Toward +x.
    PositiveX,

    /// Toward +y.
    PositiveY,

    /// Toward +z.
    PositiveZ,
}
