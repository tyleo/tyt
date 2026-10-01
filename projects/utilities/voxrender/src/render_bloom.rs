/// The halo a render adds over the emissive term before the tonemap. The
/// shader keeps each hit's emission beside its color, and the pass blurs
/// the part over the threshold into every pixel, hit or not.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBloom {
    /// The factor scaling the halo, `0..`. At `0` the pass is skipped.
    pub strength: f64,

    /// The halo's reach as a fraction of the shorter image side, `0..`
    /// exclusive: the standard deviation of the widest blur. The blur
    /// repeats at half that per octave down to one pixel, and the octaves
    /// average.
    pub radius: f64,

    /// The luminance in linear light an emission must exceed to glow,
    /// `0..`. Only the part above it blurs, hue preserved.
    pub threshold: f64,
}

/// Off, with a reach of three percent of the shorter side and a threshold
/// of `1`, which a material at glTF's default emissive strength never
/// exceeds.
impl Default for RenderBloom {
    fn default() -> Self {
        RenderBloom {
            strength: 0.0,
            radius: 0.03,
            threshold: 1.0,
        }
    }
}
