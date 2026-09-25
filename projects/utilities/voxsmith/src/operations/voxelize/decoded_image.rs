use meshdoc::MeshWrap;
use ty_math::TyVector2F64;

/// A decoded image of a mesh document: one `[r, g, b, a]` cell per texel,
/// row-major from the top-left. The bytes are color-space-neutral. The sample
/// site decodes sRGB or linear data per the slot it feeds, so one image can
/// feed slots of both kinds.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DecodedImage {
    /// Width in texels.
    pub width: u32,

    /// Height in texels.
    pub height: u32,

    /// `width * height` cells.
    pub pixels: Vec<[u8; 4]>,
}

impl DecodedImage {
    /// The nearest-neighbor texel at texture coordinate `uv` under the given
    /// wrap modes, `v` running top-down. A zero-size image yields opaque
    /// white.
    pub(crate) fn sample(&self, uv: TyVector2F64, wrap_s: MeshWrap, wrap_t: MeshWrap) -> [u8; 4] {
        if self.pixels.is_empty() {
            return [255, 255, 255, 255];
        }

        let x = wrap_texel(wrap_s, uv.x, self.width);
        let y = wrap_texel(wrap_t, uv.y, self.height);

        self.pixels[y * self.width as usize + x]
    }
}

/// The texel index for normalized `coord` along an axis of `size` texels,
/// wrapped by `wrap`.
fn wrap_texel(wrap: MeshWrap, coord: f64, size: u32) -> usize {
    let last = size as i64 - 1;
    let index = (coord * size as f64).floor() as i64;

    let wrapped = match wrap {
        MeshWrap::Repeat => index.rem_euclid(size as i64),
        MeshWrap::ClampToEdge => index.clamp(0, last),
        MeshWrap::MirroredRepeat => {
            let period = 2 * size as i64;
            let folded = index.rem_euclid(period);
            if folded < size as i64 {
                folded
            } else {
                period - 1 - folded
            }
        }
    };

    wrapped as usize
}

#[cfg(test)]
mod tests {
    use crate::operations::voxelize::DecodedImage;
    use meshdoc::MeshWrap;
    use ty_math::TyVector2F64;

    /// A 2 x 1 image, red then blue.
    fn image() -> DecodedImage {
        DecodedImage {
            width: 2,
            height: 1,
            pixels: vec![[255, 0, 0, 255], [0, 0, 255, 255]],
        }
    }

    #[test]
    fn each_wrap_maps_a_coordinate_past_one_back_onto_the_image() {
        let image = image();
        let at = |u: f64, wrap: MeshWrap| image.sample(TyVector2F64::new(u, 0.0), wrap, wrap);

        // `1.25` is a quarter into the second tile, the red texel.
        assert_eq!(at(1.25, MeshWrap::Repeat), [255, 0, 0, 255]);
        assert_eq!(at(1.25, MeshWrap::ClampToEdge), [0, 0, 255, 255]);
        // The mirrored second tile runs blue then red.
        assert_eq!(at(1.25, MeshWrap::MirroredRepeat), [0, 0, 255, 255]);
        assert_eq!(at(1.75, MeshWrap::MirroredRepeat), [255, 0, 0, 255]);
    }

    #[test]
    fn an_empty_image_samples_opaque_white() {
        let image = DecodedImage::default();
        assert_eq!(
            image.sample(TyVector2F64::ZERO, MeshWrap::Repeat, MeshWrap::Repeat),
            [255, 255, 255, 255]
        );
    }
}
