use crate::{SurfaceGrid, SurfaceMesh, SurfaceMethod, SurfaceSpan};

/// Meshes the boundary of `grid`'s solid cells by `method` with a per-cell
/// `key`. A greedy run merges only faces whose cells share a key and grows a
/// span only while `span_fits` accepts it.
pub fn mesh_grid_keyed<G: SurfaceGrid>(
    grid: &G,
    method: SurfaceMethod,
    key: &dyn Fn(G::Cell) -> u32,
    span_fits: &dyn Fn(&SurfaceSpan) -> bool,
) -> SurfaceMesh<G::Cell> {
    let bounds = grid.bounds().to_array();

    let (cull, merge) = match method {
        SurfaceMethod::Naive => (false, false),
        SurfaceMethod::Culled => (true, false),
        SurfaceMethod::Greedy => (true, true),
    };

    let mut mesh = SurfaceMesh::default();

    for d in 0..3 {
        for sign in [-1i32, 1] {
            sweep(
                grid, bounds, d, sign, cull, merge, key, span_fits, &mut mesh,
            );
        }
    }

    mesh
}

/// Sweeps the slices perpendicular to axis `d`, emitting each solid cell's
/// face on the `sign` side. `cull` drops a face whose neighbor hides it.
/// `merge` fuses the slice's exposed faces into maximal rectangles,
/// splitting where the key differs or `span_fits` refuses.
#[allow(clippy::too_many_arguments)]
fn sweep<G: SurfaceGrid>(
    grid: &G,
    bounds: [u32; 3],
    d: usize,
    sign: i32,
    cull: bool,
    merge: bool,
    key: &dyn Fn(G::Cell) -> u32,
    span_fits: &dyn Fn(&SurfaceSpan) -> bool,
    mesh: &mut SurfaceMesh<G::Cell>,
) {
    let u = (d + 1) % 3;
    let v = (d + 2) % 3;
    let w = bounds[u] as usize;
    let h = bounds[v] as usize;

    if w == 0 || h == 0 {
        return;
    }

    for s in 0..bounds[d] {
        let span = |u0, u1, v0, v1| SurfaceSpan {
            d,
            sign,
            s,
            u0,
            u1,
            v0,
            v1,
        };

        // Each exposed face carries its cell's key. An unexposed cell is
        // `None`, so merges never cross an empty gap or a key boundary.
        let mut mask = vec![None; w * h];

        for vv in 0..h {
            for uu in 0..w {
                let position = span(uu, uu + 1, vv, vv + 1).cell(uu, vv);

                let Some(cell) = grid.cell(position) else {
                    continue;
                };

                if cull {
                    let mut neighbor = position.to_array().map(i64::from);
                    neighbor[d] += i64::from(sign);

                    if let Some(neighbor) = grid.cell_at(neighbor)
                        && grid.hides(cell, neighbor)
                    {
                        continue;
                    }
                }

                mask[vv * w + uu] = Some(key(cell));
            }
        }

        if merge {
            let fits = |u0, u1, v0, v1| span_fits(&span(u0, u1, v0, v1));

            for (u0, u1, v0, v1) in merge_rects(&mask, w, h, &fits) {
                push_face(grid, mesh, &span(u0, u1, v0, v1));
            }
        } else {
            for vv in 0..h {
                for uu in 0..w {
                    if mask[vv * w + uu].is_some() {
                        push_face(grid, mesh, &span(uu, uu + 1, vv, vv + 1));
                    }
                }
            }
        }
    }
}

/// Greedily fuses the slice `mask` of width `w` and height `h` into maximal
/// rectangles of one key, each as `(u0, u1, v0, v1)` with the upper bounds
/// exclusive. Each set cell belongs to exactly one rectangle. A rectangle
/// grows only over cells that share its key and only while `fits` accepts
/// the grown rectangle.
fn merge_rects(
    mask: &[Option<u32>],
    w: usize,
    h: usize,
    fits: &dyn Fn(usize, usize, usize, usize) -> bool,
) -> Vec<(usize, usize, usize, usize)> {
    let mut consumed = vec![false; w * h];

    let mut rects = Vec::new();

    for v0 in 0..h {
        for u0 in 0..w {
            let start = v0 * w + u0;
            let Some(key) = mask[start] else {
                continue;
            };
            if consumed[start] {
                continue;
            }

            // Grow the run in +u while the cells share the key and are free.
            let mut width = 1;
            while u0 + width < w {
                let i = v0 * w + u0 + width;
                if mask[i] != Some(key) || consumed[i] || !fits(u0, u0 + width + 1, v0, v0 + 1) {
                    break;
                }
                width += 1;
            }

            // Grow in +v while every cell of the width-wide row matches.
            let mut height = 1;
            'grow: while v0 + height < h {
                for k in 0..width {
                    let i = (v0 + height) * w + u0 + k;
                    if mask[i] != Some(key) || consumed[i] {
                        break 'grow;
                    }
                }
                if !fits(u0, u0 + width, v0, v0 + height + 1) {
                    break;
                }
                height += 1;
            }

            for dy in 0..height {
                for dx in 0..width {
                    consumed[(v0 + dy) * w + u0 + dx] = true;
                }
            }

            rects.push((u0, u0 + width, v0, v0 + height));
        }
    }
    rects
}

/// Appends the quad over `span` with its corners in winding order and
/// records the cells it covers.
fn push_face<G: SurfaceGrid>(grid: &G, mesh: &mut SurfaceMesh<G::Cell>, span: &SurfaceSpan) {
    let cells = span
        .cells()
        .map(|position| grid.cell(position).expect("a quad covers solid cells"))
        .collect();

    mesh.spans.push(*span);
    mesh.face_cells.push(cells);

    let normal = span.normal();

    let base = mesh.positions.len() as u32;

    for [uu, vv] in span.corners() {
        mesh.positions.push(span.corner(uu as f32, vv as f32));
        mesh.normals.push(normal);
    }

    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[cfg(test)]
mod tests {
    use crate::{
        SurfaceMethod, SurfaceSpan, mesh_grid, mesh_grid_keyed,
        test_utilities::{MaterialGrid, live_object},
    };
    use ty_math::{TyVector3Ext, TyVector3F32, TyVector3U32};

    #[test]
    fn naive_emits_all_six_faces_per_voxel() {
        let object = live_object([1, 1, 1], &[[0, 0, 0]]);
        let mesh = mesh_grid(&object, SurfaceMethod::Naive);
        assert_eq!(mesh.quad_count(), 6);
        assert_eq!(mesh.indices.len(), 36);
        assert_eq!(mesh.positions.len(), 24);
    }

    #[test]
    fn culled_drops_the_shared_interior_faces() {
        // Two adjacent voxels: 12 faces total, the shared pair is interior.
        let object = live_object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        assert_eq!(mesh_grid(&object, SurfaceMethod::Naive).quad_count(), 12);
        assert_eq!(mesh_grid(&object, SurfaceMethod::Culled).quad_count(), 10);
    }

    #[test]
    fn a_face_against_glass_stays_and_a_seam_inside_one_glass_goes() {
        // An opaque voxel, then two of one glass: the opaque face against
        // the glass stays, the glass faces against the opaque voxel and
        // against each other go.
        let slab = MaterialGrid::new(
            [3, 1, 1],
            &[
                ([0, 0, 0], 0, true),
                ([1, 0, 0], 1, false),
                ([2, 0, 0], 1, false),
            ],
        );
        assert_eq!(mesh_grid(&slab, SurfaceMethod::Culled).quad_count(), 15);

        // Two glasses touching keep both faces of their boundary.
        let seam = MaterialGrid::new(
            [3, 1, 1],
            &[
                ([0, 0, 0], 0, true),
                ([1, 0, 0], 1, false),
                ([2, 0, 0], 2, false),
            ],
        );
        assert_eq!(mesh_grid(&seam, SurfaceMethod::Culled).quad_count(), 17);
    }

    #[test]
    fn greedy_merges_a_two_voxel_bar_into_a_box() {
        // A 2x1x1 box exposes one rectangle per face.
        let object = live_object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        assert_eq!(mesh_grid(&object, SurfaceMethod::Greedy).quad_count(), 6);
    }

    #[test]
    fn greedy_collapses_a_solid_slab() {
        // 3x3x1 solid: culled = 2*(3*3 + 3*1 + 1*3) = 30 quads; greedy = 6.
        let live: Vec<[u32; 3]> = (0..3)
            .flat_map(|x| (0..3).map(move |y| [x, y, 0]))
            .collect();
        let object = live_object([3, 3, 1], &live);
        assert_eq!(mesh_grid(&object, SurfaceMethod::Culled).quad_count(), 30);
        assert_eq!(mesh_grid(&object, SurfaceMethod::Greedy).quad_count(), 6);
    }

    #[test]
    fn keys_split_a_merged_run() {
        let object = live_object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);

        // Two keys along x, one per voxel id, split every face that spanned
        // both voxels: the two end caps stay, the four side faces each split
        // in two, for ten quads.
        let keyed = mesh_grid_keyed(
            &object,
            SurfaceMethod::Greedy,
            &|voxel_id| voxel_id.to_u32(),
            &|_| true,
        );
        assert_eq!(keyed.quad_count(), 10);
    }

    #[test]
    fn a_refused_span_stops_growing_in_either_direction() {
        // A 3x3 slab whose spans may cover two cells at most.
        let live: Vec<[u32; 3]> = (0..3)
            .flat_map(|x| (0..3).map(move |y| [x, y, 0]))
            .collect();
        let object = live_object([3, 3, 1], &live);

        let at_most_two = |span: &SurfaceSpan| (span.u1 - span.u0) * (span.v1 - span.v0) <= 2;
        let mesh = mesh_grid_keyed(&object, SurfaceMethod::Greedy, &|_| 0, &at_most_two);

        // Each 3x3 face splits into four 2x1 runs and one lone cell; each
        // 3x1 side stays a 2x1 run and a 1x1.
        assert_eq!(mesh.quad_count(), 2 * 5 + 4 * 2);
        assert!(mesh.face_cells.iter().all(|cells| cells.len() <= 2));
    }

    #[test]
    fn a_quad_records_its_span_and_the_cells_under_it() {
        let object = live_object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        let mesh = mesh_grid(&object, SurfaceMethod::Greedy);

        let top = mesh
            .spans
            .iter()
            .position(|span| span.d == 1 && span.sign > 0)
            .unwrap();

        assert_eq!(
            mesh.spans[top],
            SurfaceSpan {
                d: 1,
                sign: 1,
                s: 0,
                u0: 0,
                u1: 1,
                v0: 0,
                v1: 2,
            }
        );
        assert_eq!(
            mesh.face_cells[top],
            [
                object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap(),
                object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap(),
            ]
        );
    }

    #[test]
    fn single_voxel_spans_the_unit_cube() {
        let object = live_object([1, 1, 1], &[[0, 0, 0]]);
        let mesh = mesh_grid(&object, SurfaceMethod::Culled);
        for point in &mesh.positions {
            assert!(
                point.to_array().iter().all(|&c| c == 0.0 || c == 1.0),
                "corner {point:?}"
            );
        }
    }

    #[test]
    fn every_triangle_winds_outward() {
        // A 2x2x2 solid cube exercises all six face directions under greedy.
        let live: Vec<[u32; 3]> = (0..2)
            .flat_map(|x| (0..2).flat_map(move |y| (0..2).map(move |z| [x, y, z])))
            .collect();
        let object = live_object([2, 2, 2], &live);
        let mesh = mesh_grid(&object, SurfaceMethod::Greedy);
        assert_eq!(mesh.quad_count(), 6);

        for triangle in mesh.indices.as_chunks::<3>().0 {
            let corner = |i: u32| mesh.positions[i as usize];
            let (p0, p1, p2) = (
                corner(triangle[0]),
                corner(triangle[1]),
                corner(triangle[2]),
            );
            let stored = mesh.normals[triangle[0] as usize];
            assert!(
                TyVector3F32::triangle_normal(p0, p1, p2).dot(stored) > 0.0,
                "triangle winds inward"
            );
        }
    }
}
