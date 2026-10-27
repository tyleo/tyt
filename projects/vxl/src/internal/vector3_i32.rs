use ty_math::TyVector3I32;

/// The vector a `num_args = 3` flag parsed as `values`.
pub fn vector3_i32(values: &[i32]) -> TyVector3I32 {
    let [x, y, z] = values[..] else {
        panic!("clap takes exactly three values, got {}", values.len());
    };

    TyVector3I32::new(x, y, z)
}
