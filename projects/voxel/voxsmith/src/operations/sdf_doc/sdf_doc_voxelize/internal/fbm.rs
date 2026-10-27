use crate::operations::sdf_doc::gradient_noise;
use ty_math::TyVector3F64;

/// The standard deviation of one layer of gradient noise, measured over the
/// first 20 million points of the R3 sequence.
const SIGMA1: f64 = 0.1816;

/// The rows of the turn each layer after the first applies before doubling
/// the point. The turn keeps the layers' lattices from lining up.
const LAYER_TURN: [TyVector3F64; 3] = [
    TyVector3F64::new(0.0, 0.8, 0.6),
    TyVector3F64::new(-0.8, 0.36, -0.48),
    TyVector3F64::new(-0.6, -0.48, 0.64),
];

/// Fractal noise at `point` under `seed`, mapped into `(-1, 1)`. The noise sums
/// `octaves` layers of gradient noise, each at twice the frequency and half
/// the amplitude of the last.
pub fn fbm(point: TyVector3F64, octaves: u32, seed: u32) -> f64 {
    let mut q = point;
    let mut amplitude = 1.0;
    let mut sum = 0.0;
    let mut squared_amplitudes = 0.0;

    for octave in 0..octaves {
        if octave > 0 {
            q = TyVector3F64::new(
                LAYER_TURN[0].dot(q),
                LAYER_TURN[1].dot(q),
                LAYER_TURN[2].dot(q),
            ) * 2.0;
        }

        sum += amplitude * gradient_noise(q, seed.wrapping_add(octave));
        squared_amplitudes += amplitude * amplitude;
        amplitude *= 0.5;
    }

    let x = 0.8 * sum / (SIGMA1 * squared_amplitudes.sqrt());
    x / (1.0 + x * x).sqrt()
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{fbm, gradient_noise};
    use ty_math::TyVector3F64;

    /// The first `count` points of the R3 sequence, a low-discrepancy sequence
    /// that fills the lattice evenly, each with a seed.
    fn r3_points(count: u32) -> impl Iterator<Item = (TyVector3F64, u32)> {
        let step = TyVector3F64::new(
            0.754_877_666_246_692_7,
            0.569_840_290_998_053_2,
            0.430_159_709_001_946_8,
        );

        (0..count).map(move |index| ((step * f64::from(index)).fract() * 1024.0, index % 64))
    }

    /// The root mean square of `values`.
    fn deviation(values: impl Iterator<Item = f64>) -> f64 {
        let (sum, count) = values.fold((0.0, 0.0), |(sum, count), value| {
            (sum + value * value, count + 1.0)
        });

        (sum / count).sqrt()
    }

    #[test]
    #[ignore = "measures sigma1 over 20 million points"]
    fn print_measured_sigma1() {
        let sigma1 =
            deviation(r3_points(20_000_000).map(|(point, seed)| gradient_noise(point, seed)));
        println!("sigma1 = {sigma1}");
    }

    #[test]
    fn one_layer_spreads_to_the_deviation_the_sigmoid_expects() {
        // fbm maps one layer of noise n to x / sqrt(1 + x * x) with
        // x = 0.8 * n / sigma1, so x spreads to 0.8 when sigma1 holds.
        let spread = deviation(r3_points(200_000).map(|(point, seed)| {
            let value = fbm(point, 1, seed);
            value / (1.0 - value * value).sqrt()
        }));

        assert!((spread / 0.8 - 1.0).abs() < 0.01, "{spread}");
    }

    #[test]
    fn the_noise_stays_inside_minus_one_to_one_and_spreads() {
        let values: Vec<f64> = (0..1000)
            .map(|index| {
                let t = f64::from(index) * 0.37;
                fbm(TyVector3F64::new(t, t * 0.5, -t * 0.25), 4, 9)
            })
            .collect();

        assert!(values.iter().all(|value| value.abs() < 1.0));
        assert!(values.iter().any(|value| *value > 0.5));
        assert!(values.iter().any(|value| *value < -0.5));
    }

    #[test]
    fn the_seed_changes_the_noise() {
        let point = TyVector3F64::new(0.3, 1.7, -2.2);

        assert_ne!(fbm(point, 4, 1), fbm(point, 4, 2));
        assert_eq!(fbm(point, 4, 1), fbm(point, 4, 1));
    }
}
