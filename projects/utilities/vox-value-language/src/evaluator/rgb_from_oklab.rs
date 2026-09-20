/// Converts Oklab to linear RGB in `f64`, undoing the cube roots.
pub(crate) fn rgb_from_oklab([lightness, a, b]: [f32; 3]) -> [f32; 3] {
    let (lightness, a, b) = (f64::from(lightness), f64::from(a), f64::from(b));
    let long = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let medium = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let short = (lightness - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);

    [
        (4.076_741_662_1 * long - 3.307_711_591_3 * medium + 0.230_969_929_2 * short) as f32,
        (-1.268_438_004_6 * long + 2.609_757_401_1 * medium - 0.341_319_396_5 * short) as f32,
        (-0.004_196_086_3 * long - 0.703_418_614_7 * medium + 1.707_614_701_0 * short) as f32,
    ]
}
