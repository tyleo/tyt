use crate::{SurfaceGrid, SurfaceSpan};

/// The occlusion at each corner of `span`'s face, read in `layer` along the
/// face's axis, as [`corner_occlusion`](crate::corner_occlusion) lays it
/// out.
pub fn layer_occlusion<G: SurfaceGrid>(grid: &G, span: &SurfaceSpan, layer: i64) -> [f64; 4] {
    let (u, v) = (span.u(), span.v());

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
            grid.is_opaque_at(cell)
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
