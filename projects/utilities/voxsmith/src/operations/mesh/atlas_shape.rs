/// How the palette atlas canvas is shaped around its material texels. The texels
/// fill the top-left in row-major order and any extra area is transparent-black
/// padding the mesh never samples.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtlasShape {
    /// A single row of texels, `count` wide and one tall, with no padding.
    Line,

    /// The near-square packing that exactly holds the texels: width
    /// `ceil(sqrt(count))`, height `ceil(count / width)`.
    Fit,

    /// The smallest square holding the texels, `ceil(sqrt(count))` to a side.
    Square,

    /// The smallest square power-of-two side that holds the texels.
    Pot,

    /// An exact `side` by `side` square, rejected when too small to hold the
    /// texels.
    Exact(u32),
}
