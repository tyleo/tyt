use crate::{RenderBloom, RenderImage};
use ty_math::{TyLinSrgbF32, TyLinSrgbaF32};

/// Rec. 709's luminance weights over linear sRGB.
const LUMINANCE: TyLinSrgbF32 = TyLinSrgbF32::new(0.2126, 0.7152, 0.0722);

/// How many standard deviations out a blur's taps reach.
const TAP_REACH: f64 = 3.0;

/// Adds `bloom`'s halo over `emission` to `image` as the contract's bloom
/// section lays it out. A miss pixel the halo reaches carries the halo as
/// straight alpha, so compositing it over black gives the halo back.
///
/// # Arguments
/// * `emission` - one emissive term per pixel of `image` in row-major
///   order.
pub fn apply_bloom(image: &mut RenderImage, emission: &[TyLinSrgbF32], bloom: RenderBloom) {
    let width = image.width() as usize;
    let height = image.height() as usize;

    debug_assert_eq!(emission.len(), width * height);

    let threshold = bloom.threshold as f32;
    let bright: Vec<TyLinSrgbF32> = emission
        .iter()
        .map(|emission| over_threshold(*emission, threshold))
        .collect();

    if bright.iter().all(|color| *color == BLACK) {
        return;
    }

    let mut halo = vec![BLACK; bright.len()];
    let mut octaves = 0.0;
    let mut sigma = bloom.radius * width.min(height) as f64;

    loop {
        for (sum, blurred) in halo
            .iter_mut()
            .zip(gaussian_blur(&bright, width, height, sigma))
        {
            *sum += blurred;
        }

        octaves += 1.0;
        sigma /= 2.0;

        if sigma < 1.0 {
            break;
        }
    }

    let scale = bloom.strength as f32 / octaves;

    for y in 0..height {
        for x in 0..width {
            let halo = halo[y * width + x] * scale;

            if halo == BLACK {
                continue;
            }

            let pixel = image
                .pixel(x as u32, y as u32)
                .expect("the pixel is within the image");

            let lit = if pixel.alpha == 0.0 {
                let alpha = halo.red.max(halo.green).max(halo.blue).min(1.0);
                let color = halo / alpha;

                TyLinSrgbaF32::new(color.red, color.green, color.blue, alpha)
            } else {
                TyLinSrgbaF32::new(
                    pixel.red + halo.red,
                    pixel.green + halo.green,
                    pixel.blue + halo.blue,
                    pixel.alpha,
                )
            };

            image.set_pixel(x as u32, y as u32, lit);
        }
    }
}

const BLACK: TyLinSrgbF32 = TyLinSrgbF32::new(0.0, 0.0, 0.0);

/// The part of `emission` whose luminance exceeds `threshold`, hue
/// preserved: black at or under it.
fn over_threshold(emission: TyLinSrgbF32, threshold: f32) -> TyLinSrgbF32 {
    let luminance = emission.red * LUMINANCE.red
        + emission.green * LUMINANCE.green
        + emission.blue * LUMINANCE.blue;

    if luminance <= threshold {
        return BLACK;
    }

    emission * ((luminance - threshold) / luminance)
}

/// `source`, a `width` by `height` image in row-major order, blurred by a
/// separable Gaussian of standard deviation `sigma` pixels. Taps past the
/// edges read black.
fn gaussian_blur(
    source: &[TyLinSrgbF32],
    width: usize,
    height: usize,
    sigma: f64,
) -> Vec<TyLinSrgbF32> {
    let kernel = gaussian_kernel(sigma);
    let reach = kernel.len() / 2;

    let mut rows = vec![BLACK; source.len()];

    for y in 0..height {
        for x in 0..width {
            let mut sum = BLACK;

            for (tap, weight) in kernel.iter().enumerate() {
                if let Some(sx) = (x + tap).checked_sub(reach).filter(|sx| *sx < width) {
                    sum += source[y * width + sx] * *weight;
                }
            }

            rows[y * width + x] = sum;
        }
    }

    let mut blurred = vec![BLACK; source.len()];

    for y in 0..height {
        for x in 0..width {
            let mut sum = BLACK;

            for (tap, weight) in kernel.iter().enumerate() {
                if let Some(sy) = (y + tap).checked_sub(reach).filter(|sy| *sy < height) {
                    sum += rows[sy * width + x] * *weight;
                }
            }

            blurred[y * width + x] = sum;
        }
    }

    blurred
}

/// The weights of a Gaussian of standard deviation `sigma` pixels at each
/// offset out to [`TAP_REACH`] deviations, summing to one.
fn gaussian_kernel(sigma: f64) -> Vec<f32> {
    let reach = (TAP_REACH * sigma).ceil() as i64;

    let weights: Vec<f64> = (-reach..=reach)
        .map(|offset| {
            if offset == 0 {
                1.0
            } else {
                (-(offset * offset) as f64 / (2.0 * sigma * sigma)).exp()
            }
        })
        .collect();

    let total: f64 = weights.iter().sum();

    weights
        .into_iter()
        .map(|weight| (weight / total) as f32)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        RenderBloom, RenderImage,
        bloom::{apply_bloom, gaussian_blur, gaussian_kernel, over_threshold},
    };
    use ty_math::{TyLinSrgbF32, TyLinSrgbaF32};

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn the_part_over_the_threshold_keeps_its_hue() {
        let orange = TyLinSrgbF32::new(4.0, 2.0, 0.0);

        // Luminance 2.2808; a threshold at half of it leaves half the color.
        let over = over_threshold(orange, 1.1404);
        assert!(close(over.red, 2.0) && close(over.green, 1.0) && over.blue == 0.0);

        assert_eq!(
            over_threshold(orange, 2.29),
            TyLinSrgbF32::new(0.0, 0.0, 0.0)
        );
        assert_eq!(
            over_threshold(TyLinSrgbF32::new(0.0, 0.0, 0.0), 0.0),
            TyLinSrgbF32::new(0.0, 0.0, 0.0)
        );
        assert_eq!(over_threshold(orange, 0.0), orange);
    }

    #[test]
    fn the_kernel_sums_to_one_and_the_blur_spreads_a_point_symmetrically() {
        let kernel = gaussian_kernel(1.5);
        assert_eq!(kernel.len(), 11);
        assert!(close(kernel.iter().sum(), 1.0));
        assert_eq!(
            kernel[5],
            *kernel.iter().max_by(|a, b| a.total_cmp(b)).unwrap()
        );
        assert_eq!(kernel[3], kernel[7]);

        // A tiny deviation collapses onto the center tap.
        assert_eq!(gaussian_kernel(1e-300), [0.0, 1.0, 0.0]);

        let mut point = vec![TyLinSrgbF32::new(0.0, 0.0, 0.0); 25];
        point[12] = TyLinSrgbF32::new(1.0, 0.5, 0.0);

        let blurred = gaussian_blur(&point, 5, 5, 1.0);
        assert!(blurred[12].red < 1.0 && blurred[12].red > blurred[7].red);
        assert_eq!(blurred[7], blurred[17]);
        assert_eq!(blurred[11], blurred[13]);
        assert_eq!(blurred[7], blurred[11]);
        assert!(close(blurred[7].green, blurred[7].red / 2.0));

        // Mass past the edges is lost, so a corner point blurs to less.
        let mut corner = vec![TyLinSrgbF32::new(0.0, 0.0, 0.0); 25];
        corner[0] = TyLinSrgbF32::new(1.0, 1.0, 1.0);
        let total: f32 = gaussian_blur(&corner, 5, 5, 1.0)
            .iter()
            .map(|color| color.red)
            .sum();
        assert!(total < 0.5, "{total}");
    }

    #[test]
    fn the_halo_adds_to_hits_and_gives_misses_an_alpha() {
        let mut image = RenderImage::new(7, 1);
        image.set_pixel(3, 0, TyLinSrgbaF32::new(0.1, 0.1, 0.1, 1.0));

        let mut emission = vec![TyLinSrgbF32::new(0.0, 0.0, 0.0); 7];
        emission[3] = TyLinSrgbF32::new(8.0, 4.0, 0.0);

        apply_bloom(
            &mut image,
            &emission,
            RenderBloom {
                strength: 1.0,
                radius: 1.0,
                threshold: 1.0,
            },
        );

        // The hit keeps its alpha and brightens by the halo's center.
        let center = image.pixel(3, 0).unwrap();
        assert_eq!(center.alpha, 1.0);
        assert!(center.red > 0.1 && center.green > 0.1 && center.blue == 0.1);

        // A neighbor takes the halo's peak as its alpha and the halo over
        // that alpha as its color, so the two multiply back to the halo.
        let beside = image.pixel(2, 0).unwrap();
        assert!(beside.alpha > 0.0 && beside.alpha < 1.0, "{beside:?}");
        assert!(close(beside.red, 1.0));
        assert!(close(beside.green, 0.5));
        assert_eq!(beside.blue, 0.0);
        assert_eq!(image.pixel(4, 0), Some(beside));

        // A halo past one clamps the alpha and keeps its brightness.
        let mut bright = RenderImage::new(1, 1);
        apply_bloom(
            &mut bright,
            &[TyLinSrgbF32::new(40.0, 0.0, 0.0)],
            RenderBloom {
                strength: 1.0,
                radius: 1.0,
                threshold: 0.0,
            },
        );
        let pixel = bright.pixel(0, 0).unwrap();
        assert_eq!(pixel.alpha, 1.0);
        assert!(pixel.red > 1.0, "{pixel:?}");

        let mut dark = RenderImage::new(7, 1);
        dark.set_pixel(3, 0, TyLinSrgbaF32::new(0.1, 0.1, 0.1, 1.0));
        let before = dark.clone();
        apply_bloom(
            &mut dark,
            &emission,
            RenderBloom {
                strength: 1.0,
                radius: 1.0,
                threshold: 10.0,
            },
        );
        assert_eq!(dark, before);
    }
}
