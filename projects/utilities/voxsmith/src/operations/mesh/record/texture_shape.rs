/// The atlas canvas, counted in cells.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextureShape {
    /// The near-square packing.
    Fit,

    /// A single row of cells.
    Line,

    /// The smallest square power of two.
    Pot,

    /// The smallest square.
    Square,

    /// An exact `n`x`n` canvas of cells.
    Exact(u32),
}
