use crate::{GridSpace, MeshBaseColorMap, MeshTexture, MeshTriangle, VoxelGrid};
use ty_math::{TyLinearRgbaColorF64, TySrgbaColor, TyVector2F64, TyVector3F64, TyVector3U32};

/// Barycentric samples per grid unit of a triangle's longest edge, so a triangle
/// spanning `k` voxels is sampled about `2k` times along it.
const OVERSAMPLE: f64 = 2.0;

/// Cap on the barycentric step count per triangle, bounding the work a triangle
/// that spans much of a fine grid can demand. A cell the scatter misses under
/// this cap still resolves through the point-sample fallback.
const MAX_STEPS: usize = 16;

/// The barycentric offset that seats each lattice sample in a sub-triangle
/// interior, off the shared vertices and edges.
const THIRD: f64 = 1.0 / 3.0;

/// For each surface cell, the base color of its covering triangle's texture,
/// area-averaged. Each textured triangle is supersampled across its area and a
/// cell means the samples of the triangle that covers it, so fine texture does
/// not alias into a muddy palette. A surface cell the scatter misses (its
/// covering triangle only grazes it) point-samples that triangle at the cell
/// center, so every textured surface cell resolves. A cell whose covering
/// triangle is untextured, and every non-surface cell, is `None`.
///
/// The 8-bit re-encoding is the epsilon merge: surface points whose color rounds
/// to the same stored byte triple resolve to one palette cell.
///
/// # Arguments
/// * `triangles` - the mesh triangles, carrying texture coordinates.
/// * `base_colors` - each material's optional base-color binding, indexed by a
///   triangle's material tag.
/// * `textures` - the decoded texture table a binding indexes.
/// * `grid` - the rasterized occupancy and per-cell covering triangle.
/// * `counts` - the grid resolution in voxels per axis.
pub(crate) fn sample_base_color(
    triangles: &[MeshTriangle],
    base_colors: &[Option<MeshBaseColorMap>],
    textures: &[MeshTexture],
    grid: &VoxelGrid,
    counts: TyVector3U32,
) -> Vec<Option<TySrgbaColor>> {
    let cells = grid.filled.len();

    let Some(space) = GridSpace::from_triangles(triangles, counts) else {
        return vec![None; cells];
    };

    // A running linear-color sum and sample count per cell.
    let mut sum = vec![[0.0f64; 4]; cells];
    let mut count = vec![0u32; cells];

    for (index, triangle) in triangles.iter().enumerate() {
        let Some((uvs, map)) = textured(triangle, base_colors) else {
            continue;
        };
        let texture = &textures[map.image];

        let grids = [
            space.to_grid(triangle.points[0]),
            space.to_grid(triangle.points[1]),
            space.to_grid(triangle.points[2]),
        ];
        let steps = sample_steps(&grids);
        let inverse = 1.0 / steps as f64;

        // A regular barycentric lattice over the triangle, one point per
        // sub-triangle interior.
        for i in 0..steps {
            for j in 0..(steps - i) {
                let a = (i as f64 + THIRD) * inverse;
                let b = (j as f64 + THIRD) * inverse;
                let c = 1.0 - a - b;

                let cell = space.cell_index(bary_point(&triangle.points, a, b, c));

                // Accumulate only into the cells this triangle covers, so a
                // cell's color, finish, and point-sample fallback all come from
                // its one recorded covering triangle. A sample that floors into
                // a neighbor another triangle covers, or an interior or empty
                // cell, is dropped.
                if grid.triangle[cell] != Some(index as u32) {
                    continue;
                }

                let color = texel(texture, bary_uv(uvs, a, b, c), map);

                sum[cell][0] += color.r;
                sum[cell][1] += color.g;
                sum[cell][2] += color.b;
                sum[cell][3] += color.a;
                count[cell] += 1;
            }
        }
    }

    (0..cells)
        .map(|cell| match count[cell] {
            0 => point_sample(triangles, base_colors, textures, grid, &space, cell),
            n => Some(mean(sum[cell], n).to_srgba()),
        })
        .collect()
}

/// A surface cell the scatter missed: the color of its covering triangle's
/// texture at the cell center, or `None` when that triangle is untextured.
fn point_sample(
    triangles: &[MeshTriangle],
    base_colors: &[Option<MeshBaseColorMap>],
    textures: &[MeshTexture],
    grid: &VoxelGrid,
    space: &GridSpace,
    cell: usize,
) -> Option<TySrgbaColor> {
    let triangle = &triangles[grid.triangle[cell]? as usize];
    let (uvs, map) = textured(triangle, base_colors)?;
    let (a, b, c) = barycentric(&triangle.points, space.cell_center(cell));
    Some(texel(&textures[map.image], bary_uv(uvs, a, b, c), map).to_srgba())
}

/// A triangle's texture coordinates and base-color binding, or `None` when it
/// carries no coordinates or its material has no base-color texture.
fn textured(
    triangle: &MeshTriangle,
    base_colors: &[Option<MeshBaseColorMap>],
) -> Option<([TyVector2F64; 3], MeshBaseColorMap)> {
    Some((triangle.uvs?, base_colors[triangle.material as usize]?))
}

/// The linear color of a texel: the sampled texel decoded to linear and tinted
/// by the base-color factor.
fn texel(texture: &MeshTexture, uv: TyVector2F64, map: MeshBaseColorMap) -> TyLinearRgbaColorF64 {
    texture
        .sample(uv.x, uv.y, map.wrap_s, map.wrap_t)
        .to_linear_rgba()
        .componentwise_multiply(&map.factor)
}

/// The mean of an accumulated linear-color sum over `n` samples.
fn mean(sum: [f64; 4], n: u32) -> TyLinearRgbaColorF64 {
    let n = n as f64;
    TyLinearRgbaColorF64::new(sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n)
}

/// The barycentric step count for a triangle, tied to its longest grid-space
/// edge so a larger triangle samples more of its cells, floored at two interior
/// points and capped at [`MAX_STEPS`].
fn sample_steps(grids: &[[f64; 3]; 3]) -> usize {
    let longest = edge(grids[0], grids[1])
        .max(edge(grids[1], grids[2]))
        .max(edge(grids[2], grids[0]));

    ((longest * OVERSAMPLE).ceil() as usize).clamp(2, MAX_STEPS)
}

/// The Euclidean distance between two grid-space points.
fn edge(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// The barycentric weights of `point` projected onto a triangle's plane,
/// clamped onto the triangle so a cell center off a grazed triangle samples the
/// nearest in-triangle point. A degenerate triangle weights its first vertex.
fn barycentric(points: &[TyVector3F64; 3], point: TyVector3F64) -> (f64, f64, f64) {
    let v0 = points[1] - points[0];
    let v1 = points[2] - points[0];
    let v2 = point - points[0];

    let d00 = v0.dot(&v0);
    let d01 = v0.dot(&v1);
    let d11 = v1.dot(&v1);
    let d20 = v2.dot(&v0);
    let d21 = v2.dot(&v1);

    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < f64::EPSILON {
        return (1.0, 0.0, 0.0);
    }

    let b = (d11 * d20 - d01 * d21) / denom;
    let c = (d00 * d21 - d01 * d20) / denom;
    clamp_barycentric(1.0 - b - c, b, c)
}

/// Clamps barycentric weights back onto the triangle, renormalizing so they sum
/// to one.
fn clamp_barycentric(a: f64, b: f64, c: f64) -> (f64, f64, f64) {
    let (a, b, c) = (a.max(0.0), b.max(0.0), c.max(0.0));
    let sum = a + b + c;
    if sum > 0.0 {
        (a / sum, b / sum, c / sum)
    } else {
        (1.0, 0.0, 0.0)
    }
}

/// The point at barycentric weights `(a, b, c)` over a triangle's vertices.
fn bary_point(points: &[TyVector3F64; 3], a: f64, b: f64, c: f64) -> TyVector3F64 {
    TyVector3F64::new(
        points[0].x * a + points[1].x * b + points[2].x * c,
        points[0].y * a + points[1].y * b + points[2].y * c,
        points[0].z * a + points[1].z * b + points[2].z * c,
    )
}

/// The texture coordinate at barycentric weights `(a, b, c)` over a triangle's
/// per-vertex coordinates.
fn bary_uv(uvs: [TyVector2F64; 3], a: f64, b: f64, c: f64) -> TyVector2F64 {
    TyVector2F64::new(
        uvs[0].x * a + uvs[1].x * b + uvs[2].x * c,
        uvs[0].y * a + uvs[1].y * b + uvs[2].y * c,
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        MeshBaseColorMap, MeshTexture, MeshTriangle, MeshWrap, sample_base_color,
        voxelize_triangles,
    };
    use ty_math::{TyLinearRgbaColorF64, TySrgbaColor, TyVector2F64, TyVector3F64, TyVector3U32};

    /// Every surface cell of a textured mesh resolves to a color, through the
    /// scatter or the point-sample fallback, never staying `None`. A `None`
    /// surface cell is the grazing-miss bug that left voxels the flat white
    /// factor.
    #[test]
    fn every_textured_surface_cell_resolves() {
        // An oblique triangle so the grid rasterizes cells the barycentric
        // lattice grazes without landing a sample in.
        let triangle = MeshTriangle {
            points: [
                TyVector3F64::new(0.0, 0.0, 0.0),
                TyVector3F64::new(5.0, 1.0, 0.0),
                TyVector3F64::new(1.0, 5.0, 3.0),
            ],
            uvs: Some([
                TyVector2F64::new(0.0, 0.0),
                TyVector2F64::new(1.0, 0.0),
                TyVector2F64::new(0.0, 1.0),
            ]),
            material: 0,
        };
        let textures = vec![MeshTexture::new(
            1,
            1,
            vec![TySrgbaColor::new(255, 0, 0, 255)],
        )];
        let base_colors = vec![Some(MeshBaseColorMap {
            image: 0,
            factor: TyLinearRgbaColorF64::new(1.0, 1.0, 1.0, 1.0),
            wrap_s: MeshWrap::Repeat,
            wrap_t: MeshWrap::Repeat,
        })];
        let counts = TyVector3U32::new(8, 8, 8);
        let grid = voxelize_triangles(&[triangle], counts, false);

        let sampled = sample_base_color(&[triangle], &base_colors, &textures, &grid, counts);

        let surface = grid.triangle.iter().filter(|t| t.is_some()).count();
        assert!(surface > 0, "the oblique triangle rasterizes surface cells");
        for (cell, covering) in grid.triangle.iter().enumerate() {
            if covering.is_some() {
                assert_eq!(
                    sampled[cell],
                    Some(TySrgbaColor::new(255, 0, 0, 255)),
                    "surface cell {cell} left uncolored"
                );
            }
        }
    }
}
