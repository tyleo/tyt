use crate::{MeshTriangle, VoxelGrid, triangle_bounds, triangle_box_overlap};
use ty_math::{TyVector3F64, TyVector3U32};

/// Rasterizes a soup of material-tagged triangles into a [`VoxelGrid`], one
/// entry per cell in `x*Y*Z + y*Z + z` raster order (matching
/// [`VoxObject`](voxcore::VoxObject) voxel ids). A cell is filled when a
/// triangle passes through it, taking the material of the first triangle to
/// reach it. The mesh's bounding box is fit tightly to the grid, so the longest
/// mesh axis spans its full count, and an empty soup yields an all-empty grid.
///
/// # Arguments
/// * `triangles` - the triangles to rasterize, in grid-independent world space.
/// * `counts` - the grid resolution in voxels per axis.
/// * `solid` - when true, fill the volume the surface encloses too, by a flood
///   fill from the boundary that fills every cell the outside cannot reach; a
///   non-watertight surface leaks, so the fill falls back to the shell. Filled
///   interior cells carry no material, to be painted by the caller.
pub(crate) fn voxelize_triangles(
    triangles: &[MeshTriangle],
    counts: TyVector3U32,
    solid: bool,
) -> VoxelGrid {
    let (nx, ny, nz) = (counts.x as usize, counts.y as usize, counts.z as usize);

    let mut filled = vec![false; nx * ny * nz];

    let mut material = vec![None; nx * ny * nz];

    let points = triangles.iter().flat_map(|triangle| triangle.points);

    let Some((min, max)) = triangle_bounds(points) else {
        return VoxelGrid { filled, material };
    };

    if filled.is_empty() {
        return VoxelGrid { filled, material };
    }

    // Edge length of one voxel on each axis. A zero-extent axis (a flat mesh)
    // collapses to a single slice, so guard the divisor.
    let size = [
        voxel_size(min.x, max.x, nx),
        voxel_size(min.y, max.y, ny),
        voxel_size(min.z, max.z, nz),
    ];

    for triangle in triangles {
        // The triangle in grid coordinates, where each voxel is the unit cube
        // `[i, i + 1)` on each axis. The map onto grid space is affine and
        // affine maps preserve overlap, so the separating-axis test stays valid
        // even when `size` differs per axis.
        let grid = [
            to_grid(triangle.points[0], min, size),
            to_grid(triangle.points[1], min, size),
            to_grid(triangle.points[2], min, size),
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
                        material[cell] = Some(triangle.material);
                    }
                }
            }
        }
    }

    if solid {
        fill_enclosed(&mut filled, nx, ny, nz);
    }
    VoxelGrid { filled, material }
}

/// Fills every empty cell that the grid boundary cannot reach through empty
/// cells, so a watertight shell becomes a solid body. Flood-fills the outside
/// from the boundary, then flips whatever it never reached.
fn fill_enclosed(filled: &mut [bool], nx: usize, ny: usize, nz: usize) {
    let mut outside = vec![false; filled.len()];
    let mut stack: Vec<(usize, usize, usize)> = Vec::new();

    // Seed the flood from every empty boundary cell.
    for x in 0..nx {
        for y in 0..ny {
            for z in 0..nz {
                let on_face =
                    x == 0 || y == 0 || z == 0 || x == nx - 1 || y == ny - 1 || z == nz - 1;
                let cell = x * ny * nz + y * nz + z;
                if on_face && !filled[cell] && !outside[cell] {
                    outside[cell] = true;
                    stack.push((x, y, z));
                }
            }
        }
    }

    while let Some((x, y, z)) = stack.pop() {
        for (dx, dy, dz) in [
            (-1i32, 0, 0),
            (1, 0, 0),
            (0, -1, 0),
            (0, 1, 0),
            (0, 0, -1),
            (0, 0, 1),
        ] {
            let (a, b, c) = (x as i32 + dx, y as i32 + dy, z as i32 + dz);

            if a < 0 || b < 0 || c < 0 || a as usize >= nx || b as usize >= ny || c as usize >= nz {
                continue;
            }

            let (a, b, c) = (a as usize, b as usize, c as usize);

            let cell = a * ny * nz + b * nz + c;

            if !filled[cell] && !outside[cell] {
                outside[cell] = true;
                stack.push((a, b, c));
            }
        }
    }

    // Any cell the outside flood never reached is enclosed: fill it.
    for (cell, fill) in filled.iter_mut().enumerate() {
        if !outside[cell] {
            *fill = true;
        }
    }
}

/// The edge length of one voxel on an axis, or `1.0` for a zero-extent axis.
fn voxel_size(min: f64, max: f64, count: usize) -> f64 {
    let extent = max - min;
    if extent > 0.0 && count > 0 {
        extent / count as f64
    } else {
        1.0
    }
}

/// A point in grid coordinates, where one unit is one voxel from the grid's min
/// corner.
fn to_grid(point: TyVector3F64, min: TyVector3F64, size: [f64; 3]) -> [f64; 3] {
    [
        (point.x - min.x) / size[0],
        (point.y - min.y) / size[1],
        (point.z - min.z) / size[2],
    ]
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

/// A floored grid coordinate clamped to `0..=last`.
fn clamp_index(value: f64, last: usize) -> usize {
    if value < 0.0 {
        0
    } else {
        (value as usize).min(last)
    }
}

#[cfg(test)]
mod tests {
    use crate::{MeshTriangle, voxelize_triangles};
    use ty_math::{TyVector3F64, TyVector3U32};

    /// Tags a triangle soup with one material so the geometry tests can build
    /// [`MeshTriangle`]s from plain points.
    fn tagged(points: Vec<[[f64; 3]; 3]>, material: u32) -> Vec<MeshTriangle> {
        points
            .into_iter()
            .map(|points| MeshTriangle {
                points: points.map(|[x, y, z]| TyVector3F64::new(x, y, z)),
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
        // Every filled cell is a surface cell, so all carry a material.
        assert!(grid.material.iter().filter(|m| m.is_some()).count() == 64 - 8);
    }

    #[test]
    fn solid_fills_the_enclosed_volume_leaving_interior_material_unset() {
        let grid = voxelize_triangles(&cube(4.0), TyVector3U32::new(4, 4, 4), true);
        assert_eq!(live_count(&grid.filled), 64);
        assert!(grid.filled.iter().all(|&filled| filled));
        // The 2^3 interior cells fill with no material; the 56 shell cells keep
        // theirs.
        assert_eq!(grid.material.iter().filter(|m| m.is_some()).count(), 64 - 8);
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
            solid.material[interior], None,
            "an interior cell carries no material"
        );
    }

    #[test]
    fn a_surface_cell_takes_the_first_covering_triangle_material() {
        // Two quads cover the one cell of a 1x1x1 grid; the first drawn, material
        // 7, wins over the later material 9.
        let first = tagged(vec![[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]], 7);
        let second = tagged(vec![[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]]], 9);
        let triangles: Vec<_> = first.into_iter().chain(second).collect();
        let grid = voxelize_triangles(&triangles, TyVector3U32::new(1, 1, 1), false);
        assert_eq!(grid.material, vec![Some(7)]);
    }

    #[test]
    fn empty_soup_yields_empty_grid() {
        let grid = voxelize_triangles(&[], TyVector3U32::new(2, 2, 2), true);
        assert_eq!(live_count(&grid.filled), 0);
        assert!(grid.material.iter().all(|m| m.is_none()));
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
