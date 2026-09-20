/// Converts linear RGB to Oklab in `f64` through the LMS cube roots.
pub(crate) fn oklab_from_rgb([red, green, blue]: [f32; 3]) -> [f32; 3] {
    let (red, green, blue) = (f64::from(red), f64::from(green), f64::from(blue));
    let long = (0.412_221_470_8 * red + 0.536_332_536_3 * green + 0.051_445_992_9 * blue).cbrt();
    let medium = (0.211_903_498_2 * red + 0.680_699_545_1 * green + 0.107_396_956_6 * blue).cbrt();
    let short = (0.088_302_461_9 * red + 0.281_718_837_6 * green + 0.629_978_700_5 * blue).cbrt();

    [
        (0.210_454_255_3 * long + 0.793_617_785_0 * medium - 0.004_072_046_8 * short) as f32,
        (1.977_998_495_1 * long - 2.428_592_205_0 * medium + 0.450_593_709_9 * short) as f32,
        (0.025_904_037_1 * long + 0.782_771_766_2 * medium - 0.808_675_766_0 * short) as f32,
    ]
}
