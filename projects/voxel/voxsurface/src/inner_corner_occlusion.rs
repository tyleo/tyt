use crate::{SurfaceGrid, SurfaceSpan, layer_occlusion};

/// How open each corner of `span`'s face is seen from inside the cells it
/// covers: [`corner_occlusion`](crate::corner_occlusion) read in the span's
/// own layer instead of the layer the face looks into.
pub fn inner_corner_occlusion<G: SurfaceGrid>(grid: &G, span: &SurfaceSpan) -> [f64; 4] {
    layer_occlusion(grid, span, i64::from(span.s))
}

#[cfg(test)]
mod tests {
    use crate::{
        SurfaceSpan, corner_occlusion, inner_corner_occlusion, test_utilities::MaterialGrid,
    };

    #[test]
    fn a_face_seen_from_inside_reads_its_own_layer() {
        // A glass cell beside an opaque one, both on the grid's floor.
        let pair = MaterialGrid::new([3, 3, 3], &[([0, 0, 0], 0, false), ([1, 0, 0], 1, true)]);

        let top = SurfaceSpan {
            d: 2,
            sign: 1,
            s: 0,
            u0: 0,
            u1: 1,
            v0: 0,
            v1: 1,
        };

        // The opaque cell closes a third at the corners along x = 1, from
        // inside only.
        assert_eq!(
            inner_corner_occlusion(&pair, &top),
            [1.0, 2.0 / 3.0, 2.0 / 3.0, 1.0]
        );
        assert_eq!(corner_occlusion(&pair, &top), [1.0; 4]);

        // The bottom face lies on the grid's edge and still reads its layer.
        let bottom = SurfaceSpan { sign: -1, ..top };
        assert_eq!(
            inner_corner_occlusion(&pair, &bottom),
            [1.0, 1.0, 2.0 / 3.0, 2.0 / 3.0]
        );
    }
}
