use crate::GRID_DIRECTION_BITS;
use ty_math::TyVector3F64;

/// `vector` scaled so its largest component is `1 << GRID_DIRECTION_BITS` in
/// magnitude, then rounded.
pub fn quantize_direction(vector: TyVector3F64) -> [i32; 3] {
    let largest = vector.abs().max_element();

    if largest == 0.0 {
        return [0; 3];
    }

    let scale = f64::from(1u32 << GRID_DIRECTION_BITS) / largest;

    vector
        .to_array()
        .map(|component| (component * scale).round() as i32)
}

#[cfg(test)]
mod tests {
    use crate::quantize_direction;
    use ty_math::TyVector3F64;

    #[test]
    fn the_largest_component_lands_on_the_bound() {
        assert_eq!(
            quantize_direction(TyVector3F64::new(0.5, -1.0, 0.25)),
            [32768, -65536, 16384]
        );
        assert_eq!(quantize_direction(TyVector3F64::ZERO), [0; 3]);
    }
}
