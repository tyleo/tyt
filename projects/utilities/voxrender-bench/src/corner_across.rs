use voxrender::{GRID_FRACTION_BITS, SHADOW_INSET_BITS};

/// One cell's side in fixed point.
const ONE: i64 = 1 << GRID_FRACTION_BITS;

/// Where corner `corner`'s shadow ray starts across a face from its min
/// corner along its `u` and `v`, in fixed point. Bit 1 of `corner` picks the
/// far edge on `u`, bit 2 on `v`.
pub fn corner_across(corner: u32) -> [i64; 2] {
    let inset = ONE >> SHADOW_INSET_BITS;
    let edge = |far: bool| if far { ONE - inset } else { inset };

    [edge(corner & 1 != 0), edge(corner & 2 != 0)]
}

#[cfg(test)]
mod tests {
    use crate::corner_across;

    #[test]
    fn each_corner_insets_from_its_edges() {
        assert_eq!(corner_across(0), [8, 8]);
        assert_eq!(corner_across(1), [8184, 8]);
        assert_eq!(corner_across(2), [8, 8184]);
        assert_eq!(corner_across(3), [8184, 8184]);
    }
}
