/// A PNG's channel format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PngChannels {
    Grey,
    GreyAlpha,
    Rgb,
    Rgba,
}

impl PngChannels {
    /// The samples per texel.
    pub fn count(self) -> usize {
        match self {
            PngChannels::Grey => 1,
            PngChannels::GreyAlpha => 2,
            PngChannels::Rgb => 3,
            PngChannels::Rgba => 4,
        }
    }
}
