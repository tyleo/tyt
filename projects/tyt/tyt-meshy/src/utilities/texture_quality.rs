use clap::ValueEnum;

/// The resolution of the generated base color texture.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum TextureQuality {
    /// Normal-quality base color texture.
    Normal,

    /// HD (4096×4096) base color texture. Only supported on Meshy 6.
    Hd,
}

impl TextureQuality {
    /// Returns the value sent as the API's `hd_texture`.
    pub fn is_hd(self) -> bool {
        matches!(self, TextureQuality::Hd)
    }
}
