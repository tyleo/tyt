use ty_math::TyVector3F64;

/// The vector a `num_args = 3` flag parsed as `values`.
pub fn vector3_f64(values: &[f64]) -> TyVector3F64 {
    let [x, y, z] = values[..] else {
        panic!("clap takes exactly three values, got {}", values.len());
    };

    TyVector3F64::new(x, y, z)
}
