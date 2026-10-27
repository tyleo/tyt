use ty_math::TySrgbaU8;
use voxcore::color::lin_srgba_f64_from_srgba_u8;

/// The linear-light components of an `[r, g, b]` byte color.
pub fn color_floats(color: [u8; 3]) -> [f64; 3] {
    let [red, green, blue] = color;
    let linear = lin_srgba_f64_from_srgba_u8(TySrgbaU8::new(red, green, blue, 255));
    [linear.red, linear.green, linear.blue]
}
