use crate::{SurfaceGrid, SurfaceSpan};

/// How open each corner of `span`'s face is, in the order of its
/// [`corners`](SurfaceSpan::corners): `1` fully open. Each of the three
/// cells beside the corner in the layer the face looks into closes a third.
/// Both cells along the face's edges together close it fully. So does a
/// solid cell over the corner. Only a naive mesh emits a face under one.
pub fn corner_occlusion<G: SurfaceGrid>(grid: &G, span: &SurfaceSpan) -> [f64; 4] {
    let (u, v) = (span.u(), span.v());

    let layer = i64::from(span.s) + i64::from(span.sign);

    span.corners().map(|[uu, vv]| {
        // Along each tangent axis, the cell under the face and the cell
        // beside the corner outside it.
        let along = |at: usize, start: usize| {
            let at = at as i64;
            if at == start as i64 {
                (at, at - 1)
            } else {
                (at - 1, at)
            }
        };
        let (under_u, beside_u) = along(uu, span.u0);
        let (under_v, beside_v) = along(vv, span.v0);

        let solid = |at_u: i64, at_v: i64| {
            let mut cell = [0i64; 3];
            cell[span.d] = layer;
            cell[u] = at_u;
            cell[v] = at_v;
            grid.is_solid(cell)
        };

        let over = solid(under_u, under_v);
        let side_u = solid(beside_u, under_v);
        let side_v = solid(under_u, beside_v);
        let diagonal = solid(beside_u, beside_v);

        let open = if over || (side_u && side_v) {
            0
        } else {
            3 - u8::from(side_u) - u8::from(side_v) - u8::from(diagonal)
        };

        f64::from(open) / 3.0
    })
}

#[cfg(test)]
mod tests {
    use crate::{SurfaceSpan, corner_occlusion, test_utilities::live_object};

    /// The top face of the cell at `(x, y, 0)`.
    fn top(x: usize, y: usize) -> SurfaceSpan {
        SurfaceSpan {
            d: 2,
            sign: 1,
            s: 0,
            u0: x,
            u1: x + 1,
            v0: y,
            v1: y + 1,
        }
    }

    #[test]
    fn a_neighbor_beside_a_corner_closes_a_third_and_two_close_it() {
        // The top of (0,0,0) meets (1,0,1) along its x = 1 edge, so the two
        // corners at x = 1 close a third and the two at x = 0 stay open.
        let step = live_object([3, 3, 3], &[[0, 0, 0], [1, 0, 0], [1, 0, 1]]);
        assert_eq!(
            corner_occlusion(&step, &top(0, 0)),
            [1.0, 2.0 / 3.0, 2.0 / 3.0, 1.0]
        );

        // An inner corner: the top of (0,0,0) meets (1,0,1) and (0,1,1) at
        // its (1,1) corner. The diagonal cell (1,1,1) is empty.
        let inner = live_object([3, 3, 3], &[[0, 0, 0], [1, 0, 1], [0, 1, 1]]);
        assert_eq!(
            corner_occlusion(&inner, &top(0, 0)),
            [1.0, 2.0 / 3.0, 0.0, 2.0 / 3.0]
        );

        // A diagonal alone closes a third.
        let diagonal = live_object([3, 3, 3], &[[0, 0, 0], [1, 1, 1]]);
        assert_eq!(
            corner_occlusion(&diagonal, &top(0, 0)),
            [1.0, 1.0, 2.0 / 3.0, 1.0]
        );
    }

    #[test]
    fn a_merged_span_reads_its_outer_corners() {
        // A bar of three along x with a cell on its third voxel: the top
        // face merged over the first two closes at its x = 2 corners only.
        let bar = live_object([3, 3, 3], &[[0, 0, 0], [1, 0, 0], [2, 0, 0], [2, 0, 1]]);
        let span = SurfaceSpan { u1: 2, ..top(0, 0) };
        assert_eq!(
            corner_occlusion(&bar, &span),
            [1.0, 2.0 / 3.0, 2.0 / 3.0, 1.0]
        );
    }

    #[test]
    fn a_corner_under_a_solid_cell_is_closed() {
        let pair = live_object([3, 3, 3], &[[0, 0, 0], [0, 0, 1]]);
        assert_eq!(corner_occlusion(&pair, &top(0, 0)), [0.0; 4]);
    }
}
