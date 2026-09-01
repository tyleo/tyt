use crate::{
    GridSpace, MeshBaseColorMap, MeshEmissiveMap, MeshMaterial, MeshMaterialMaps,
    MeshMetallicRoughnessMap, MeshOcclusionMap, MeshSampler, MeshTexture, MeshTriangle, VoxelGrid,
};
use ty_math::{TyLinSrgbaF64, TySrgbaU8, TyVector2F64, TyVector3F64, TyVector3U32};
use voxcore::color::lin_srgba_f64_from_srgba_u8;

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

/// For each surface cell, its covering triangle's material with every present PBR
/// map sampled from the covering material's textures, area-averaged over the
/// cell's footprint. Each textured triangle is supersampled across its area and a
/// cell means the samples of the triangle that covers it, so fine texture does
/// not alias into a muddy palette. An attribute whose map is absent keeps the
/// material's flat factor. A surface cell the scatter misses (its covering
/// triangle only grazes it) point-samples that triangle at the cell center, so
/// every surface cell resolves; a non-surface cell is `None`.
///
/// The color space is chosen per map: base color and emissive decode sRGB, while
/// metallic-roughness and occlusion are straight linear data. The sampled
/// attributes merge on their float bit patterns downstream.
///
/// # Arguments
/// * `triangles` - the mesh triangles, carrying per-map texture coordinates.
/// * `materials` - the flat per-material factors, indexed by a triangle's tag,
///   the floor each cell's sampled attributes override.
/// * `maps` - each material's optional texture bindings, parallel to `materials`.
/// * `textures` - the decoded texture table a binding indexes.
/// * `grid` - the rasterized occupancy and per-cell covering triangle.
/// * `counts` - the grid resolution in voxels per axis.
pub(crate) fn sample_material(
    triangles: &[MeshTriangle],
    materials: &[MeshMaterial],
    maps: &[MeshMaterialMaps],
    textures: &[MeshTexture],
    grid: &VoxelGrid,
    counts: TyVector3U32,
) -> Vec<Option<MeshMaterial>> {
    let cells = grid.filled.len();

    let Some(space) = GridSpace::from_triangles(triangles, counts) else {
        return vec![None; cells];
    };

    // A running per-attribute sum and shared sample count per cell.
    let mut accum = vec![CellAccum::default(); cells];

    for (index, triangle) in triangles.iter().enumerate() {
        let material_maps = &maps[triangle.material_index as usize];
        if !samplable(triangle, material_maps) {
            continue;
        }

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
                // cell's attributes, and its point-sample fallback, all come from
                // its one recorded covering triangle. A sample that floors into a
                // neighbor another triangle covers, or an interior or empty cell,
                // is dropped.
                if grid.triangle[cell] != Some(index as u32) {
                    continue;
                }

                accumulate(&mut accum[cell], triangle, material_maps, textures, a, b, c);
            }
        }
    }

    (0..cells)
        .map(|cell| {
            resolve_cell(
                triangles,
                materials,
                maps,
                textures,
                grid,
                &space,
                &accum[cell],
                cell,
            )
        })
        .collect()
}

/// The resolved material of one cell: `None` for a non-surface cell, else the
/// covering triangle's flat material with each present, coordinated map applied
/// as the footprint mean (when the scatter covered the cell) or a point sample at
/// the cell center (when it grazed).
#[allow(clippy::too_many_arguments)]
fn resolve_cell(
    triangles: &[MeshTriangle],
    materials: &[MeshMaterial],
    maps: &[MeshMaterialMaps],
    textures: &[MeshTexture],
    grid: &VoxelGrid,
    space: &GridSpace,
    scatter: &CellAccum,
    cell: usize,
) -> Option<MeshMaterial> {
    let covering = grid.triangle[cell]? as usize;
    let triangle = &triangles[covering];
    let material_maps = &maps[triangle.material_index as usize];
    let mut material = materials[triangle.material_index as usize];

    let accum = if scatter.count > 0 {
        *scatter
    } else if samplable(triangle, material_maps) {
        point_accum(triangle, material_maps, textures, space, cell)
    } else {
        return Some(material);
    };

    apply(&mut material, triangle, material_maps, &accum);

    Some(material)
}

/// A per-cell running sum of each sampled attribute over `count` samples.
#[derive(Clone, Copy, Default)]
struct CellAccum {
    /// The linear base color, summed component-wise.
    base_color: [f64; 4],

    /// The metallic value, summed.
    metallic: f64,

    /// The roughness value, summed.
    roughness: f64,

    /// The linear emissive color, summed component-wise.
    emissive: [f64; 3],

    /// The occlusion value, summed.
    occlusion: f64,

    /// The number of samples contributing to the sums.
    count: u32,
}

/// Whether a triangle has at least one map to sample, a binding paired with the
/// coordinates its slot needs.
fn samplable(triangle: &MeshTriangle, maps: &MeshMaterialMaps) -> bool {
    (maps.base_color.is_some() && triangle.uvs.base_color.is_some())
        || (maps.metallic_roughness.is_some() && triangle.uvs.metallic_roughness.is_some())
        || (maps.emissive.is_some() && triangle.uvs.emissive.is_some())
        || (maps.occlusion.is_some() && triangle.uvs.occlusion.is_some())
}

/// Samples each present, coordinated map at barycentric `(a, b, c)` and adds it
/// to `accum`, counting one sample.
fn accumulate(
    accum: &mut CellAccum,
    triangle: &MeshTriangle,
    maps: &MeshMaterialMaps,
    textures: &[MeshTexture],
    a: f64,
    b: f64,
    c: f64,
) {
    if let (Some(map), Some(uvs)) = (maps.base_color, triangle.uvs.base_color) {
        let color = base_color_linear(texture(textures, map.sampler), bary_uv(uvs, a, b, c), &map);
        accum.base_color[0] += color[0];
        accum.base_color[1] += color[1];
        accum.base_color[2] += color[2];
        accum.base_color[3] += color[3];
    }

    if let (Some(map), Some(uvs)) = (maps.metallic_roughness, triangle.uvs.metallic_roughness) {
        let (metallic, roughness) =
            metallic_roughness(texture(textures, map.sampler), bary_uv(uvs, a, b, c), &map);
        accum.metallic += metallic;
        accum.roughness += roughness;
    }

    if let (Some(map), Some(uvs)) = (maps.emissive, triangle.uvs.emissive) {
        let color = emissive(texture(textures, map.sampler), bary_uv(uvs, a, b, c), &map);
        accum.emissive[0] += color[0];
        accum.emissive[1] += color[1];
        accum.emissive[2] += color[2];
    }

    if let (Some(map), Some(uvs)) = (maps.occlusion, triangle.uvs.occlusion) {
        accum.occlusion += occlusion(texture(textures, map.sampler), bary_uv(uvs, a, b, c), &map);
    }

    accum.count += 1;
}

/// One point sample at the cell center, for a surface cell the scatter grazed
/// without a lattice point landing in it.
fn point_accum(
    triangle: &MeshTriangle,
    maps: &MeshMaterialMaps,
    textures: &[MeshTexture],
    space: &GridSpace,
    cell: usize,
) -> CellAccum {
    let (a, b, c) = barycentric(&triangle.points, space.cell_center(cell));
    let mut accum = CellAccum::default();
    accumulate(&mut accum, triangle, maps, textures, a, b, c);
    accum
}

/// Overrides each present, coordinated attribute of `material` with its
/// accumulated mean. An attribute whose map is absent keeps its flat factor.
fn apply(
    material: &mut MeshMaterial,
    triangle: &MeshTriangle,
    maps: &MeshMaterialMaps,
    accum: &CellAccum,
) {
    let n = accum.count as f64;

    if maps.base_color.is_some() && triangle.uvs.base_color.is_some() {
        material.base_color = TyLinSrgbaF64::new(
            accum.base_color[0] / n,
            accum.base_color[1] / n,
            accum.base_color[2] / n,
            accum.base_color[3] / n,
        );
    }

    if maps.metallic_roughness.is_some() && triangle.uvs.metallic_roughness.is_some() {
        material.metallic = accum.metallic / n;
        material.roughness = accum.roughness / n;
    }

    // The map overrides the emissive color; the emissive strength stays the
    // material's flat factor, since it is a per-material scalar the texture does
    // not carry.
    if maps.emissive.is_some() && triangle.uvs.emissive.is_some() {
        material.emissive_color = TyLinSrgbaF64::new(
            accum.emissive[0] / n,
            accum.emissive[1] / n,
            accum.emissive[2] / n,
            1.0,
        );
    }

    if maps.occlusion.is_some() && triangle.uvs.occlusion.is_some() {
        material.occlusion = accum.occlusion / n;
    }
}

/// The linear base color of a texel: the sRGB-decoded texel tinted by the linear
/// factor.
fn base_color_linear(texture: &MeshTexture, uv: TyVector2F64, map: &MeshBaseColorMap) -> [f64; 4] {
    let texel = texture.sample(uv.x, uv.y, map.sampler.wrap_s, map.sampler.wrap_t);
    let color = lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(texel)) * map.factor;
    [color.red, color.green, color.blue, color.alpha]
}

/// The metallic and roughness of a texel: straight-decoded linear data, blue
/// scaled by the metallic factor and green by the roughness factor.
fn metallic_roughness(
    texture: &MeshTexture,
    uv: TyVector2F64,
    map: &MeshMetallicRoughnessMap,
) -> (f64, f64) {
    let texel = texture.sample(uv.x, uv.y, map.sampler.wrap_s, map.sampler.wrap_t);
    let data = texel.map(|byte| byte as f64 / 255.0);
    (map.metallic * data[2], map.roughness * data[1])
}

/// The linear emissive color of a texel: the sRGB-decoded texel tinted
/// component-wise by the linear emissive factor.
fn emissive(texture: &MeshTexture, uv: TyVector2F64, map: &MeshEmissiveMap) -> [f64; 3] {
    let texel = texture.sample(uv.x, uv.y, map.sampler.wrap_s, map.sampler.wrap_t);
    let color = lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(texel));
    [
        color.red * map.factor[0],
        color.green * map.factor[1],
        color.blue * map.factor[2],
    ]
}

/// The occlusion of a texel: straight-decoded red at `1 + strength * (red - 1)`,
/// so `strength` scales how far the map darkens from full.
fn occlusion(texture: &MeshTexture, uv: TyVector2F64, map: &MeshOcclusionMap) -> f64 {
    let red = texture.sample(uv.x, uv.y, map.sampler.wrap_s, map.sampler.wrap_t)[0] as f64 / 255.0;
    1.0 + map.strength * (red - 1.0)
}

/// The texture a sampler indexes.
fn texture(textures: &[MeshTexture], sampler: MeshSampler) -> &MeshTexture {
    &textures[sampler.image]
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

/// The barycentric weights of `point` projected onto a triangle's plane, clamped
/// onto the triangle so a cell center off a grazed triangle samples the nearest
/// in-triangle point. A degenerate triangle weights its first vertex.
fn barycentric(points: &[TyVector3F64; 3], point: TyVector3F64) -> (f64, f64, f64) {
    let v0 = points[1] - points[0];
    let v1 = points[2] - points[0];
    let v2 = point - points[0];

    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);

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
        MeshBaseColorMap, MeshMaterial, MeshMaterialMaps, MeshSampler, MeshTexture, MeshTriangle,
        MeshTriangleUvs, MeshWrap, sample_material, voxelize_triangles,
    };
    use ty_math::{TyLinSrgbaF64, TyVector2F64, TyVector3F64, TyVector3U32};

    /// Every surface cell of a textured mesh resolves to a material, through the
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
            uvs: MeshTriangleUvs {
                base_color: Some([
                    TyVector2F64::new(0.0, 0.0),
                    TyVector2F64::new(1.0, 0.0),
                    TyVector2F64::new(0.0, 1.0),
                ]),
                ..Default::default()
            },
            material_index: 0,
        };
        let materials = vec![MeshMaterial::flat(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0))];
        let textures = vec![MeshTexture::new(1, 1, vec![[255, 0, 0, 255]])];
        let maps = vec![MeshMaterialMaps {
            base_color: Some(MeshBaseColorMap {
                sampler: MeshSampler {
                    image: 0,
                    wrap_s: MeshWrap::Repeat,
                    wrap_t: MeshWrap::Repeat,
                },
                factor: TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0),
            }),
            ..Default::default()
        }];
        let counts = TyVector3U32::new(8, 8, 8);
        // Triangle-cover, hollow: the covering array a texel sampler reads.
        let grid = voxelize_triangles(&[triangle], counts, false, false);

        let sampled = sample_material(&[triangle], &materials, &maps, &textures, &grid, counts);

        let surface = grid.triangle.iter().filter(|t| t.is_some()).count();
        assert!(surface > 0, "the oblique triangle rasterizes surface cells");
        for (cell, covering) in grid.triangle.iter().enumerate() {
            if covering.is_some() {
                assert_eq!(
                    sampled[cell].map(|material| material.base_color),
                    Some(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0)),
                    "surface cell {cell} left uncolored"
                );
            }
        }
    }
}
