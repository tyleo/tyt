use ty_math::TyLinSrgbF32;

/// One pixel of a render: the light that reached it and its transmittance,
/// the share of what lies behind the scene that passes through per channel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPixel {
    /// The light, in linear radiance.
    pub light: TyLinSrgbF32,

    /// The transmittance, each channel `0..1`.
    pub transmittance: TyLinSrgbF32,
}

/// A miss: no light and full transmittance.
impl Default for RenderPixel {
    fn default() -> Self {
        RenderPixel {
            light: TyLinSrgbF32::new(0.0, 0.0, 0.0),
            transmittance: TyLinSrgbF32::new(1.0, 1.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::RenderPixel;
    use ty_math::TyLinSrgbF32;

    #[test]
    fn the_default_pixel_is_a_miss() {
        let pixel = RenderPixel::default();

        assert_eq!(pixel.light, TyLinSrgbF32::new(0.0, 0.0, 0.0));
        assert_eq!(pixel.transmittance, TyLinSrgbF32::new(1.0, 1.0, 1.0));
    }
}
