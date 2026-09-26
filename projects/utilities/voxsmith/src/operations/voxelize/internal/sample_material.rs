use crate::operations::voxelize::{
    GridSpace, MeshInput, MeshTriangle, TextureSlots, VoxelGrid, VoxelMaterial,
};
use meshdoc::MeshMaterial;
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TySrgbaU8, TyVector3F64};
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

/// For each surface cell, its covering triangle's material with every resolved
/// texture slot sampled from the material's textures, area-averaged over the
/// cell's footprint. Each textured triangle is supersampled across its area and
/// a cell means the samples of the triangle that covers it, so fine texture
/// does not alias into a muddy palette. An attribute whose slot is unresolved
/// keeps the material's flat factor. A surface cell the scatter misses (its
/// covering triangle only grazes it) point-samples that triangle at the cell
/// center, so every surface cell resolves; a non-surface cell is `None`.
///
/// The color space is chosen per slot: base color and emissive decode sRGB,
/// while metallic-roughness and occlusion are straight linear data. The
/// sampled attributes merge on their float bit patterns downstream.
///
/// # Arguments
/// * `mesh` - the flattened mesh the triangles' materials are read from.
/// * `triangles` - the triangles `grid` was rasterized from.
/// * `grid` - the rasterized occupancy and per-cell covering triangle.
/// * `space` - the grid `grid` was rasterized onto.
pub(crate) fn sample_material(
    mesh: &MeshInput<'_>,
    triangles: &[MeshTriangle],
    grid: &VoxelGrid,
    space: &GridSpace,
) -> Vec<Option<VoxelMaterial>> {
    let cells = grid.filled.len();

    let slots: Vec<TextureSlots<'_>> = (0..mesh.primitives.len())
        .map(|index| TextureSlots::resolve(mesh, index as u32))
        .collect();

    // A running per-attribute sum and shared sample count per cell.
    let mut accum = vec![CellAccum::default(); cells];

    for (index, triangle) in triangles.iter().enumerate() {
        let slots = &slots[triangle.primitive as usize];
        if !slots.any() {
            continue;
        }

        let material = mesh.primitives[triangle.primitive as usize].material;

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

                accumulate(&mut accum[cell], triangle, material, slots, a, b, c);
            }
        }
    }

    (0..cells)
        .map(|cell| resolve_cell(mesh, triangles, &slots, grid, space, &accum[cell], cell))
        .collect()
}

/// The resolved material of one cell: `None` for a non-surface cell, else the
/// covering triangle's flat material with each resolved slot applied as the
/// footprint mean (when the scatter covered the cell) or a point sample at the
/// cell center (when it grazed).
fn resolve_cell(
    mesh: &MeshInput<'_>,
    triangles: &[MeshTriangle],
    slots: &[TextureSlots<'_>],
    grid: &VoxelGrid,
    space: &GridSpace,
    scatter: &CellAccum,
    cell: usize,
) -> Option<VoxelMaterial> {
    let covering = grid.triangle[cell]? as usize;
    let triangle = &triangles[covering];
    let slots = &slots[triangle.primitive as usize];
    let source = mesh.primitives[triangle.primitive as usize].material;
    let mut material = VoxelMaterial::from(source);

    let accum = if scatter.count > 0 {
        *scatter
    } else if slots.any() {
        point_accum(triangle, source, slots, space, cell)
    } else {
        return Some(material);
    };

    apply(&mut material, slots, &accum);

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

/// Samples each resolved slot at barycentric `(a, b, c)` over `triangle`,
/// applies the factor of `material`, and adds it to `accum` as one sample.
fn accumulate(
    accum: &mut CellAccum,
    triangle: &MeshTriangle,
    material: &MeshMaterial,
    slots: &TextureSlots<'_>,
    a: f64,
    b: f64,
    c: f64,
) {
    let vertex_ids = triangle.vertex_ids;

    // The sRGB-decoded texel tinted by the linear factor.
    if let Some(slot) = &slots.base_color {
        let texel = slot.sample(vertex_ids, a, b, c);
        let color =
            lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(texel)) * material.base_color_factor;
        accum.base_color[0] += color.red;
        accum.base_color[1] += color.green;
        accum.base_color[2] += color.blue;
        accum.base_color[3] += color.alpha;
    }

    // Straight-decoded linear data, blue scaled by the metallic factor and
    // green by the roughness factor.
    if let Some(slot) = &slots.metallic_roughness {
        let data = slot
            .sample(vertex_ids, a, b, c)
            .map(|byte| byte as f64 / 255.0);
        accum.metallic += material.metallic_factor * data[2];
        accum.roughness += material.roughness_factor * data[1];
    }

    // The sRGB-decoded texel tinted component-wise by the linear emissive
    // factor.
    if let Some(slot) = &slots.emissive {
        let texel = slot.sample(vertex_ids, a, b, c);
        let color = lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(texel));
        let factor = material.emissive_factor;
        accum.emissive[0] += color.red * factor.red;
        accum.emissive[1] += color.green * factor.green;
        accum.emissive[2] += color.blue * factor.blue;
    }

    // Straight-decoded red at `1 + strength * (red - 1)`, so the strength
    // scales how far the map darkens from full.
    if let Some(slot) = &slots.occlusion {
        let red = slot.sample(vertex_ids, a, b, c)[0] as f64 / 255.0;
        accum.occlusion += 1.0 + material.occlusion_strength * (red - 1.0);
    }

    accum.count += 1;
}

/// One point sample at the cell center, for a surface cell the scatter grazed
/// without a lattice point landing in it.
fn point_accum(
    triangle: &MeshTriangle,
    material: &MeshMaterial,
    slots: &TextureSlots<'_>,
    space: &GridSpace,
    cell: usize,
) -> CellAccum {
    let (a, b, c) = barycentric(&triangle.points, space.cell_center(cell));
    let mut accum = CellAccum::default();
    accumulate(&mut accum, triangle, material, slots, a, b, c);
    accum
}

/// Overrides each resolved slot's attribute of `material` with its accumulated
/// mean. An attribute whose slot is unresolved keeps its flat factor.
fn apply(material: &mut VoxelMaterial, slots: &TextureSlots<'_>, accum: &CellAccum) {
    let n = accum.count as f64;

    if slots.base_color.is_some() {
        material.base_color = TyLinSrgbaF64::new(
            accum.base_color[0] / n,
            accum.base_color[1] / n,
            accum.base_color[2] / n,
            accum.base_color[3] / n,
        );
    }

    if slots.metallic_roughness.is_some() {
        material.metallic = accum.metallic / n;
        material.roughness = accum.roughness / n;
    }

    // The map overrides the emissive color; the emissive strength stays the
    // material's flat factor, since it is a per-material scalar the texture does
    // not carry.
    if slots.emissive.is_some() {
        material.emissive_color = TyLinSrgbF64::new(
            accum.emissive[0] / n,
            accum.emissive[1] / n,
            accum.emissive[2] / n,
        );
    }

    if slots.occlusion.is_some() {
        material.occlusion = accum.occlusion / n;
    }
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

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        dependencies::DependenciesImpl,
        operations::voxelize::{
            GridSpace, VoxelFrame, VoxelScale, document_of, mesh_input_from_mesh_main, png_rgba,
            sample_material, triangle_of, voxelize_triangles,
        },
    };
    use branded_id::U32Id;
    use meshdoc::{
        MeshImage, MeshImageMediaType, MeshImageSource, MeshMain, MeshMaterial, MeshPrimitive,
        MeshTexture, MeshTextureRef,
    };
    use ty_math::{TyLinSrgbaF64, TyTransformF64, TyVector2F64, TyVector3F64, TyVector3U32};

    /// Every surface cell of a textured mesh resolves to a material, through the
    /// scatter or the point-sample fallback, never staying `None`. A `None`
    /// surface cell is the grazing-miss bug that left voxels the flat white
    /// factor.
    #[test]
    fn every_textured_surface_cell_resolves() {
        let mut main = MeshMain::default();
        let image_id = main
            .retain_image(MeshImage {
                name: String::new(),
                media_type: MeshImageMediaType::Png,
                source: MeshImageSource::Bytes(png_rgba(1, 1, &[[255, 0, 0, 255]])),
            })
            .unwrap();
        let texture_id = main.retain_texture(MeshTexture::new(image_id)).unwrap();
        let material_id = main
            .retain_material(MeshMaterial {
                base_color_texture: Some(MeshTextureRef {
                    texture_id,
                    uv_stream_id: U32Id::from_u32(0),
                }),
                ..Default::default()
            })
            .unwrap();

        // An oblique triangle so the grid rasterizes cells the barycentric
        // lattice grazes without landing a sample in.
        let mut primitive = MeshPrimitive::new(
            vec![
                TyVector3F64::new(0.0, 0.0, 0.0),
                TyVector3F64::new(5.0, 1.0, 0.0),
                TyVector3F64::new(1.0, 5.0, 3.0),
            ],
            vec![triangle_of(0, 1, 2)],
        )
        .unwrap();
        primitive
            .push_uv_stream(vec![
                TyVector2F64::new(0.0, 0.0),
                TyVector2F64::new(1.0, 0.0),
                TyVector2F64::new(0.0, 1.0),
            ])
            .unwrap();
        primitive.set_material_id(Some(material_id));
        let document = document_of(main, primitive, None, TyTransformF64::default());

        let mesh = mesh_input_from_mesh_main(
            &DependenciesImpl,
            &document,
            VoxelFrame::World,
            VoxelScale::Bake,
        )
        .unwrap();
        let bounds = mesh.object_bounds(&mesh.objects[0], None).unwrap();
        let space = GridSpace::on_lattice(&bounds, TyVector3F64::splat(0.625));
        assert_eq!(space.counts(), TyVector3U32::new(8, 8, 5));
        // Triangle-cover, hollow: the covering array a texel sampler reads.
        let grid = voxelize_triangles(&mesh.triangles, &space, false, false);

        let sampled = sample_material(&mesh, &mesh.triangles, &grid, &space);

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
