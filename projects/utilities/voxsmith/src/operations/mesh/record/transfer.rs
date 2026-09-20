/// The transfer a written value declares.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transfer {
    /// Applies no transfer.
    Linear,

    /// Applies the sRGB transfer.
    Srgb,
}
