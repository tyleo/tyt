use crate::{
    GridSpace, MeshTriangle, VoxelGrid, clamp_index, triangle_bounds, triangle_box_overlap,
};
use ty_math::TyVector3U32;

/// Rasterizes a soup of material-tagged triangles into a [`VoxelGrid`], one
/// entry per cell in `x*Y*Z + y*Z + z` raster order (matching
/// [`VoxObject`](voxcore::VoxObject) voxel ids). A cell is filled when a
/// triangle passes through it, recording the first triangle to reach it. The
/// mesh's bounding box is fit tightly to the grid, so the longest mesh axis
/// spans its full count, and an empty soup yields an all-empty grid.
///
/// # Arguments
/// * `triangles` - the triangles to rasterize, in grid-independent world space.
/// * `counts` - the grid resolution in voxels per axis.
/// * `solid` - when true, occupancy becomes the body the surface bounds, every
///   voxel whose center lies inside it, replacing the surface shell; see
///   [`fill_center_inside`]. A non-watertight surface leaks. Filled interior
///   cells record no triangle, to be painted by the caller.
pub(crate) fn voxelize_triangles(
    triangles: &[MeshTriangle],
    counts: TyVector3U32,
    solid: bool,
) -> VoxelGrid {
    let (nx, ny, nz) = (counts.x as usize, counts.y as usize, counts.z as usize);

    let mut filled = vec![false; nx * ny * nz];

    let mut covering = vec![None; nx * ny * nz];

    let points = triangles.iter().flat_map(|triangle| triangle.points);

    // No triangles, or a zero-size grid: an all-empty grid.
    let Some((min, max)) = triangle_bounds(points).filter(|_| !filled.is_empty()) else {
        return VoxelGrid {
            filled,
            triangle: covering,
        };
    };

    // The affine map onto grid space, where each voxel is the unit cube
    // `[i, i + 1)` on each axis. A zero-extent axis (a flat mesh) collapses to a
    // single slice, guarded inside the map.
    let space = GridSpace::from_bounds(min, max, counts);

    for (index, triangle) in triangles.iter().enumerate() {
        // The triangle in grid coordinates. The map onto grid space is affine
        // and affine maps preserve overlap, so the separating-axis test stays
        // valid even when the per-axis voxel size differs.
        let grid = [
            space.to_grid(triangle.points[0]),
            space.to_grid(triangle.points[1]),
            space.to_grid(triangle.points[2]),
        ];

        let (lo, hi) = cell_range(&grid, [nx, ny, nz]);

        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    let cell = x * ny * nz + y * nz + z;

                    if filled[cell] {
                        continue;
                    }

                    let center = [x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5];

                    if triangle_box_overlap(center, 0.5, &grid) {
                        filled[cell] = true;
                        covering[cell] = Some(index as u32);
                    }
                }
            }
        }
    }

    if solid {
        // Replace the surface occupancy with the body the surface bounds: every
        // voxel whose center lies inside it. The surface pass above still stands
        // as `covering`, so a filled boundary cell keeps the triangle it samples.
        fill_center_inside(&mut filled, &space, triangles, nx, ny, nz);
    }

    VoxelGrid {
        filled,
        triangle: covering,
    }
}

/// Marks every voxel whose center lies inside the surface, so a solid fill is
/// the body the mesh bounds, not its shell. Per column along x it stabs a ray
/// through the column center and fills a cell when an odd number of triangle
/// crossings lie past its center: the even-odd rule for a point in a closed
/// surface.
///
/// Counting crossings is winding independent, unlike a normal-facing test, so a
/// mesh with mixed or inward winding still fills correctly. A face lying exactly
/// on a cell boundary is a non-issue: the test asks only whether a center, at the
/// voxel midpoint, is enclosed, and a midpoint never lands on an axis-aligned
/// boundary plane. So a one-voxel gap between two bodies stays empty, where a
/// boundary-inclusive surface raster fills it from both flanking walls.
///
/// The ray is nudged a hair off the cell lattice on the two cross axes, by
/// distinct offsets, so it never grazes a shared edge or the diagonal splitting a
/// square face into two triangles, either of which would double count. A
/// watertight surface stabs an even count per column; an open one may stab an odd
/// count and fill past its last crossing, the documented leak of a non-watertight
/// solid fill.
fn fill_center_inside(
    filled: &mut [bool],
    space: &GridSpace,
    triangles: &[MeshTriangle],
    nx: usize,
    ny: usize,
    nz: usize,
) {
    // Off-lattice on y and z so a column ray never grazes a lattice-aligned edge;
    // the two offsets differ so a square face's dividing diagonal, which runs
    // where y and z advance together, resolves to exactly one of its triangles.
    const OFFSET_Y: f64 = 1.0e-3;
    const OFFSET_Z: f64 = 3.0e-3;

    // The surface pass left the shell, and its boundary over-marking, in
    // `filled`. Clear it so the body is exactly what the ray stab encloses;
    // every boundary cell whose center is inside is refilled below.
    filled.iter_mut().for_each(|filled| *filled = false);

    // Each column's crossing x's, indexed `y * nz + z`. Scattering a triangle
    // into only the columns its y-z projection covers keeps the stab near-linear
    // in triangles, not one pass over every triangle per column.
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); ny * nz];

    for triangle in triangles {
        let grid = [
            space.to_grid(triangle.points[0]),
            space.to_grid(triangle.points[1]),
            space.to_grid(triangle.points[2]),
        ];

        let Some((y_first, y_last)) =
            column_span([grid[0][1], grid[1][1], grid[2][1]], OFFSET_Y, ny)
        else {
            continue;
        };
        let Some((z_first, z_last)) =
            column_span([grid[0][2], grid[1][2], grid[2][2]], OFFSET_Z, nz)
        else {
            continue;
        };

        for y in y_first..=y_last {
            for z in z_first..=z_last {
                let ray_y = y as f64 + 0.5 + OFFSET_Y;
                let ray_z = z as f64 + 0.5 + OFFSET_Z;
                if let Some(x) = column_crossing(&grid, ray_y, ray_z) {
                    columns[y * nz + z].push(x);
                }
            }
        }
    }

    for y in 0..ny {
        for z in 0..nz {
            let crossings = &mut columns[y * nz + z];

            // A column needs an entry and an exit to enclose any cell.
            if crossings.len() < 2 {
                continue;
            }

            crossings.sort_by(|a, b| a.total_cmp(b));

            for x in 0..nx {
                let center = x as f64 + 0.5;
                let before = crossings
                    .iter()
                    .filter(|&&crossing| crossing < center)
                    .count();
                if before % 2 == 1 {
                    filled[x * ny * nz + y * nz + z] = true;
                }
            }
        }
    }
}

/// The inclusive range of column indices on one cross axis whose center-line ray,
/// at `index + 0.5 + offset`, can fall within the triangle's span there, clamped
/// to `0..count`. `None` when the span lies wholly outside the grid, so a
/// triangle scatters only into the columns it can cross.
fn column_span(values: [f64; 3], offset: f64, count: usize) -> Option<(usize, usize)> {
    let low = values[0].min(values[1]).min(values[2]);
    let high = values[0].max(values[1]).max(values[2]);

    // ray(index) in [low, high] => index in
    // [low - 0.5 - offset, high - 0.5 - offset].
    let first = (low - 0.5 - offset).ceil();
    let last = (high - 0.5 - offset).floor();

    let ceiling = (count - 1) as f64;
    if last < 0.0 || first > ceiling || first > last {
        return None;
    }

    Some((first.max(0.0) as usize, last.min(ceiling) as usize))
}

/// The grid-space x at which the ray through `(ray_y, ray_z)` cast along x
/// crosses `grid`, or `None` when the column misses the triangle or the triangle
/// runs parallel to the ray. Solves the triangle's barycentric coordinates at
/// `(ray_y, ray_z)` in the y-z plane, then interpolates x at that point.
fn column_crossing(grid: &[[f64; 3]; 3], ray_y: f64, ray_z: f64) -> Option<f64> {
    let [a, b, c] = grid;

    // The y-z edge matrix determinant; near zero for a face parallel to the ray,
    // which a column never crosses transversally.
    let determinant = (b[1] - a[1]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[1] - a[1]);
    if determinant.abs() <= f64::EPSILON {
        return None;
    }

    let offset_y = ray_y - a[1];
    let offset_z = ray_z - a[2];

    let u = (offset_y * (c[2] - a[2]) - offset_z * (c[1] - a[1])) / determinant;
    let v = ((b[1] - a[1]) * offset_z - (b[2] - a[2]) * offset_y) / determinant;
    if u < 0.0 || v < 0.0 || u + v > 1.0 {
        return None;
    }

    let w = 1.0 - u - v;
    Some(w * a[0] + u * b[0] + v * c[0])
}

/// The inclusive voxel-index box a grid-space triangle can touch, clamped to
/// the grid.
fn cell_range(grid: &[[f64; 3]; 3], counts: [usize; 3]) -> ([usize; 3], [usize; 3]) {
    let mut lo = [0usize; 3];
    let mut hi = [0usize; 3];

    for (axis, (low, high)) in lo.iter_mut().zip(hi.iter_mut()).enumerate() {
        let a = grid[0][axis];
        let b = grid[1][axis];
        let c = grid[2][axis];
        let last = counts[axis].saturating_sub(1);

        *low = clamp_index(a.min(b).min(c).floor(), last);
        *high = clamp_index(a.max(b).max(c).floor(), last);
    }

    (lo, hi)
}

#[cfg(test)]
mod tests {
    use crate::{MeshTriangle, MeshTriangleUvs, voxelize_triangles};
    use ty_math::{TyVector3F64, TyVector3U32};

    /// Tags a triangle soup with one material so the geometry tests can build
    /// [`MeshTriangle`]s from plain points.
    fn tagged(points: Vec<[[f64; 3]; 3]>, material: u32) -> Vec<MeshTriangle> {
        points
            .into_iter()
            .map(|points| MeshTriangle {
                points: points.map(|[x, y, z]| TyVector3F64::new(x, y, z)),
                uvs: MeshTriangleUvs::default(),
                material,
            })
            .collect()
    }

    /// The 12 triangles of an axis-aligned cube spanning `[0, edge]` on each
    /// axis, all material `0`.
    fn cube(edge: f64) -> Vec<MeshTriangle> {
        let v = [
            [0.0, 0.0, 0.0],
            [edge, 0.0, 0.0],
            [edge, edge, 0.0],
            [0.0, edge, 0.0],
            [0.0, 0.0, edge],
            [edge, 0.0, edge],
            [edge, edge, edge],
            [0.0, edge, edge],
        ];
        let faces = [
            [0, 1, 2],
            [0, 2, 3],
            [4, 6, 5],
            [4, 7, 6],
            [0, 4, 5],
            [0, 5, 1],
            [3, 2, 6],
            [3, 6, 7],
            [0, 3, 7],
            [0, 7, 4],
            [1, 5, 6],
            [1, 6, 2],
        ];
        tagged(
            faces.iter().map(|&[a, b, c]| [v[a], v[b], v[c]]).collect(),
            0,
        )
    }

    fn live_count(occupancy: &[bool]) -> usize {
        occupancy.iter().filter(|&&filled| filled).count()
    }

    #[test]
    fn surface_of_a_cube_is_a_hollow_shell() {
        let grid = voxelize_triangles(&cube(4.0), TyVector3U32::new(4, 4, 4), false);
        // A 4^3 grid with a one-voxel-thick shell: 4^3 - 2^3.
        assert_eq!(live_count(&grid.filled), 64 - 8);
        // Every filled cell is a surface cell, so all record a triangle.
        assert!(grid.triangle.iter().filter(|t| t.is_some()).count() == 64 - 8);
    }

    #[test]
    fn solid_fills_the_enclosed_volume_leaving_interior_triangle_unset() {
        let grid = voxelize_triangles(&cube(4.0), TyVector3U32::new(4, 4, 4), true);
        assert_eq!(live_count(&grid.filled), 64);
        assert!(grid.filled.iter().all(|&filled| filled));
        // The 2^3 interior cells fill with no triangle; the 56 shell cells keep
        // theirs.
        assert_eq!(grid.triangle.iter().filter(|t| t.is_some()).count(), 64 - 8);
    }

    #[test]
    fn raster_order_places_surface_and_interior_cells() {
        // Raster index x*Y*Z + y*Z + z, the same order VoxObject voxel ids use.
        let grid_size = TyVector3U32::new(8, 8, 8);
        let corner = 0; // (0, 0, 0), on the shell
        let interior = 3 * 64 + 3 * 8 + 3; // (3, 3, 3), well inside

        let surface = voxelize_triangles(&cube(8.0), grid_size, false);
        assert!(surface.filled[corner], "the origin corner is on the shell");
        assert!(
            !surface.filled[interior],
            "an interior cell is hollow in surface mode"
        );

        let solid = voxelize_triangles(&cube(8.0), grid_size, true);
        assert!(
            solid.filled[interior],
            "an interior cell fills in solid mode"
        );
        assert_eq!(
            solid.triangle[interior], None,
            "an interior cell records no triangle"
        );
    }

    #[test]
    fn a_surface_cell_records_the_first_covering_triangle() {
        // Two quads cover the one cell of a 1x1x1 grid; the first drawn, triangle
        // 0, wins over the later triangle 1.
        let first = tagged(vec![[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]], 7);
        let second = tagged(vec![[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]]], 9);
        let triangles: Vec<_> = first.into_iter().chain(second).collect();
        let grid = voxelize_triangles(&triangles, TyVector3U32::new(1, 1, 1), false);
        assert_eq!(grid.triangle, vec![Some(0)]);
    }

    /// A cube spanning `[min, min + edge]` on each axis, offset from the origin,
    /// so a test can place several with gaps between them.
    fn cube_at(min: [f64; 3], edge: f64) -> Vec<MeshTriangle> {
        cube(edge)
            .into_iter()
            .map(|mut triangle| {
                triangle.points = triangle.points.map(|point| {
                    TyVector3F64::new(point.x + min[0], point.y + min[1], point.z + min[2])
                });
                triangle
            })
            .collect()
    }

    #[test]
    fn a_one_voxel_gap_between_two_solids_stays_empty() {
        // Two unit cubes at x cells 0 and 2 leave a one-voxel gap at cell 1. A
        // solid fill must preserve it: the cube faces sit exactly on the grid
        // planes, so a boundary-inclusive raster would fill the gap from both
        // walls and merge the pair into a solid bar. This is the mesh -> voxelize
        // round trip of two spaced voxels.
        let mut triangles = cube_at([0.0, 0.0, 0.0], 1.0);
        triangles.extend(cube_at([2.0, 0.0, 0.0], 1.0));

        let grid = voxelize_triangles(&triangles, TyVector3U32::new(3, 1, 1), true);

        assert_eq!(grid.filled, vec![true, false, true]);
    }

    #[test]
    fn empty_soup_yields_empty_grid() {
        let grid = voxelize_triangles(&[], TyVector3U32::new(2, 2, 2), true);
        assert_eq!(live_count(&grid.filled), 0);
        assert!(grid.triangle.iter().all(|t| t.is_none()));
    }

    #[test]
    fn a_flat_axis_collapses_to_one_slice() {
        // A quad in the z = 0 plane has zero extent on z, so the grid is one
        // voxel deep there and never divides by zero.
        let quad = tagged(
            vec![
                [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 4.0, 0.0]],
                [[0.0, 0.0, 0.0], [4.0, 4.0, 0.0], [0.0, 4.0, 0.0]],
            ],
            0,
        );
        let grid = voxelize_triangles(&quad, TyVector3U32::new(4, 4, 1), false);
        assert_eq!(live_count(&grid.filled), 16);
    }
}
