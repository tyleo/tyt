use crate::{
    BASE_COLOR_FACTOR, EMISSIVE_FACTOR, EMISSIVE_STRENGTH, Error, FillMode, IOR, METALLIC_FACTOR,
    MaterialMode, Mesh, MeshMaterial, OCCLUSION_STRENGTH, ROUGHNESS_FACTOR, Result, SurfaceMode,
    TRANSMISSION_FACTOR, VoxelGrid, sample_material, voxelize_triangles,
};
use branded_id::U32Id;
use std::collections::{HashMap, VecDeque};
use ty_math::{TySrgbaU8, TyTransformF64, TyVector3, TyVector3U32};
use voxcore::{
    BVoxMaterial, BVoxValuePoolValue, VoxBound, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
    VoxValuePool,
};

/// The color a body with no sampled surface falls back to when `fill_color` is
/// `None`: opaque white. Held as bytes since palette's `Srgba` has no const
/// constructor; wrap it with `TySrgbaU8::from` at each use site.
const DEFAULT_FILL: [u8; 4] = [255, 255, 255, 255];

/// Voxelizes a [`Mesh`] into a [`VoxMain`] of one object placed by one root
/// node. Errors when the mesh has no triangle geometry or the grid exceeds
/// voxcore's dense-grid limit.
///
/// # Arguments
/// * `mesh` - the mesh to rasterize, in Z-up world space.
/// * `counts` - grid resolution in voxels per axis, sized by the caller from the
///   mesh extent (see [`Mesh::extent`]).
/// * `surface_mode` - center-inside occupancy or triangle-cover; see
///   [`SurfaceMode`].
/// * `fill_mode` - whether the interior is filled or the result is hollow.
/// * `material_mode` - the color source: per-primitive flat factors, per-texel
///   base-color sampling, `flat`, or `auto` (per-texel when the mesh is
///   textured, else per-primitive).
/// * `fill_color` - the color of voxels a mode cannot sample, or `None` for the
///   `none` default.
/// * `node_scale` - the placing node's uniform scale.
/// * `name` - object-name override; `None` uses the mesh's own name.
/// * `fallback_name` - the object name when neither `name` nor the mesh has one.
#[allow(clippy::too_many_arguments)]
pub fn voxelize_mesh(
    mesh: &Mesh,
    counts: TyVector3U32,
    surface_mode: SurfaceMode,
    fill_mode: FillMode,
    material_mode: MaterialMode,
    fill_color: Option<[u8; 4]>,
    node_scale: f64,
    name: Option<&str>,
    fallback_name: &str,
) -> Result<VoxMain> {
    if mesh.triangles.is_empty() {
        return Err(Error::invalid("mesh has no triangle geometry"));
    }

    // Cap the grid before rasterizing, so an oversized resolution errors rather
    // than overflowing or exhausting memory allocating the occupancy grid.
    let volume = counts.x as u64 * counts.y as u64 * counts.z as u64;
    if volume > VoxObject::MAX_GRID_CELLS {
        return Err(grid_too_large(counts));
    }

    let grid = voxelize_triangles(
        &mesh.triangles,
        counts,
        surface_mode == SurfaceMode::CenterInside,
        fill_mode == FillMode::Solid,
    );

    let cell_materials = resolve_materials(mesh, &grid, counts, material_mode, fill_color);

    let mut state = VoxMain::default();

    let (palette, samples, default_material) = build_palette(&mut state, &cell_materials)?;

    let palette_id = state.add_palette(palette)?;

    // The object name: an explicit override, else the mesh's own name, else the
    // caller's fallback.
    let object_name = name
        .map(str::to_owned)
        .or(mesh.name.clone())
        .unwrap_or_else(|| fallback_name.to_owned());

    let mut object = VoxObject::new(object_name, counts).map_err(|_| grid_too_large(counts))?;

    object.add_layer(palette_id, default_material);

    for (index, sample) in samples.iter().enumerate() {
        if let Some(material) = sample {
            let voxel = U32Id::from_u32(index as u32);
            object
                .retain_voxel(voxel, &[*material])
                .expect("a grid index is a live voxel sampling the one layer");
        }
    }

    let object_id = state.add_object(object)?;

    // One root node placing the object and carrying the real-world scale.
    let transform = TyTransformF64 {
        scale: TyVector3::splat(node_scale),
        ..Default::default()
    };

    let node = VoxHierarchyNode {
        child_object_ids: vec![object_id],
        transform,
        ..Default::default()
    };

    let node_id = state.add_hierarchy_node(node)?;

    state.push_root_hierarchy_node_id(node_id)?;

    Ok(state)
}

/// The material of every filled cell, per the color mode: `flat` paints one fill
/// color, `per-primitive` reads each covering material's flat factors,
/// `per-texel` samples the covering material's maps per texel, and `auto` samples
/// a textured mesh and reads factors otherwise. Under the surface modes a `solid`
/// interior then takes the fill color or its nearest surface cell's material.
fn resolve_materials(
    mesh: &Mesh,
    grid: &VoxelGrid,
    counts: TyVector3U32,
    material_mode: MaterialMode,
    fill_color: Option<[u8; 4]>,
) -> Vec<Option<MeshMaterial>> {
    let mut cell_materials = match material_mode {
        MaterialMode::Flat => return flat_cells(grid, fill_color),

        MaterialMode::PerPrimitive => primitive_cells(mesh, grid),

        MaterialMode::PerTexel => sampled_cells(mesh, grid, counts),

        MaterialMode::Auto if mesh.is_textured() => sampled_cells(mesh, grid, counts),

        MaterialMode::Auto => primitive_cells(mesh, grid),
    };

    fill_interior(grid, counts, fill_color, &mut cell_materials);

    // The surface pass records a covering triangle on every cell a face passes
    // through, including a boundary-grazed cell a solid fill leaves outside its
    // enclosed body. Occupancy is `grid.filled`, so drop a material on any cell
    // the body does not fill, or a one-voxel gap between two solids would still
    // emit its over-marked walls as voxels.
    for (cell, material) in cell_materials.iter_mut().enumerate() {
        if grid.filled[cell] {
            continue;
        }
        *material = None;
    }

    cell_materials
}

/// Every filled cell takes the one fill color (white when `none`).
fn flat_cells(grid: &VoxelGrid, fill_color: Option<[u8; 4]>) -> Vec<Option<MeshMaterial>> {
    let material = MeshMaterial::flat(fill_srgba(fill_color));
    grid.filled
        .iter()
        .map(|&filled| filled.then_some(material))
        .collect()
}

/// Each surface cell takes its covering triangle's flat material; interior and
/// empty cells are `None`.
fn primitive_cells(mesh: &Mesh, grid: &VoxelGrid) -> Vec<Option<MeshMaterial>> {
    grid.triangle
        .iter()
        .map(|&covering| {
            covering
                .map(|triangle| mesh.materials[mesh.triangles[triangle as usize].material as usize])
        })
        .collect()
}

/// Each surface cell takes its covering material with every present map sampled
/// per texel over the cell footprint; interior and empty cells are `None`.
fn sampled_cells(mesh: &Mesh, grid: &VoxelGrid, counts: TyVector3U32) -> Vec<Option<MeshMaterial>> {
    sample_material(
        &mesh.triangles,
        &mesh.materials,
        &mesh.maps,
        &mesh.textures,
        grid,
        counts,
    )
}

/// Paints every filled interior cell (a `solid` body's invented volume, carrying
/// no surface material): the fill color when one is given, else the material of
/// its nearest surface cell. A solid-enclosed interior always reaches a surface
/// cell, so the white fallback is a defensive guard.
fn fill_interior(
    grid: &VoxelGrid,
    counts: TyVector3U32,
    fill_color: Option<[u8; 4]>,
    cell_materials: &mut [Option<MeshMaterial>],
) {
    let has_interior = grid
        .filled
        .iter()
        .zip(&grid.triangle)
        .any(|(&filled, triangle)| filled && triangle.is_none());

    if !has_interior {
        return;
    }

    match fill_color {
        Some([r, g, b, a]) => {
            let fill = MeshMaterial::flat(TySrgbaU8::new(r, g, b, a));
            for (cell, triangle) in grid.triangle.iter().enumerate() {
                if grid.filled[cell] && triangle.is_none() {
                    cell_materials[cell] = Some(fill);
                }
            }
        }
        None => {
            let nearest = nearest_surface_cell(grid, counts);
            for cell in 0..grid.filled.len() {
                if grid.filled[cell] && grid.triangle[cell].is_none() {
                    let resolved = nearest[cell]
                        .and_then(|source| cell_materials[source])
                        .unwrap_or_else(|| MeshMaterial::flat(TySrgbaU8::from(DEFAULT_FILL)));
                    cell_materials[cell] = Some(resolved);
                }
            }
        }
    }
}

/// A built palette, each filled cell's material sample, and the default
/// material.
type PaletteBuild = (
    VoxPalette,
    Vec<Option<U32Id<BVoxMaterial>>>,
    U32Id<BVoxMaterial>,
);

/// Assembles a palette from a per-cell material list. Near-identical
/// materials merge to one palette material, and each filled cell samples its
/// cell's material. Every glTF property draws from a deduplicated value pool
/// added to `state`: a property carrying one value id per material.
/// The default material is the first built, or a lone white material for an
/// all-empty grid, so the palette is never empty.
fn build_palette(
    state: &mut VoxMain,
    cell_materials: &[Option<MeshMaterial>],
) -> Result<PaletteBuild> {
    // Merge near-identical materials into a distinct list, first seen in raster
    // order, remembering each filled cell's position in it.
    let mut distinct: Vec<MeshMaterial> = Vec::new();
    let mut lookup: HashMap<MaterialKey, usize> = HashMap::new();
    let cell_indices: Vec<Option<usize>> = cell_materials
        .iter()
        .map(|&material| {
            material.map(|material| {
                *lookup.entry(material_key(&material)).or_insert_with(|| {
                    let index = distinct.len();
                    distinct.push(material);
                    index
                })
            })
        })
        .collect();

    // An all-empty grid still needs a non-empty palette so its pools and default
    // material are valid; give it a lone white material.
    if distinct.is_empty() {
        distinct.push(MeshMaterial::flat(TySrgbaU8::from(DEFAULT_FILL)));
    }

    // One deduplicated pool per property, plus each distinct material's
    // value id into it. The bounded scalars clamp to their pool range so the
    // pools build.
    let base_color = srgba_pool(&distinct, |material| material.base_color);
    let metallic = float_pool(&distinct, |material| material.metallic.clamp(0.0, 1.0));
    let roughness = float_pool(&distinct, |material| material.roughness.clamp(0.0, 1.0));
    let emissive_factor = srgb_pool(&distinct, |material| material.emissive_factor);
    let emissive_strength = float_pool(&distinct, |material| material.emissive_strength.max(0.0));
    let occlusion = float_pool(&distinct, |material| material.occlusion.clamp(0.0, 1.0));
    let ior = float_pool(&distinct, |material| material.ior.max(1.0));
    let transmission = float_pool(&distinct, |material| material.transmission.clamp(0.0, 1.0));

    // Register the pools and add each property. All properties precede any
    // material, so no material carries a back-fill placeholder value id.
    let base_color_pool = state.add_value_pool(VoxValuePool::srgba(base_color.values)?);
    let metallic_pool = state.add_value_pool(bounded_float(metallic.values, 0.0, 1.0)?);
    let roughness_pool = state.add_value_pool(bounded_float(roughness.values, 0.0, 1.0)?);
    let emissive_factor_pool = state.add_value_pool(VoxValuePool::srgb(emissive_factor.values)?);
    let emissive_strength_pool = state.add_value_pool(float_above(emissive_strength.values, 0.0)?);
    let occlusion_pool = state.add_value_pool(bounded_float(occlusion.values, 0.0, 1.0)?);
    let ior_pool = state.add_value_pool(float_above(ior.values, 1.0)?);
    let transmission_pool = state.add_value_pool(bounded_float(transmission.values, 0.0, 1.0)?);

    let mut palette = VoxPalette::default();
    palette
        .add_property(
            BASE_COLOR_FACTOR.to_owned(),
            base_color_pool,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .add_property(
            METALLIC_FACTOR.to_owned(),
            metallic_pool,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .add_property(
            ROUGHNESS_FACTOR.to_owned(),
            roughness_pool,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .add_property(
            EMISSIVE_FACTOR.to_owned(),
            emissive_factor_pool,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .add_property(
            EMISSIVE_STRENGTH.to_owned(),
            emissive_strength_pool,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .add_property(
            OCCLUSION_STRENGTH.to_owned(),
            occlusion_pool,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .add_property(IOR.to_owned(), ior_pool, U32Id::from_u32(0))
        .expect("the property names are distinct");
    palette
        .add_property(
            TRANSMISSION_FACTOR.to_owned(),
            transmission_pool,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");

    // One material per distinct mesh material, its value ids in property
    // order.
    let materials: Vec<U32Id<BVoxMaterial>> = (0..distinct.len())
        .map(|index| {
            palette
                .add_material(vec![
                    base_color.indices[index],
                    metallic.indices[index],
                    roughness.indices[index],
                    emissive_factor.indices[index],
                    emissive_strength.indices[index],
                    occlusion.indices[index],
                    ior.indices[index],
                    transmission.indices[index],
                ])
                .expect("one value id for each property")
        })
        .collect();

    let samples = cell_indices
        .iter()
        .map(|&index| index.map(|index| materials[index]))
        .collect();

    let default_material = materials[0];

    Ok((palette, samples, default_material))
}

/// A deduplicated pool column and each distinct material's value id into it.
struct PoolColumn<T> {
    /// The distinct values, in first-seen order.
    values: Vec<T>,

    /// Per distinct material, its value id into [`values`](Self::values).
    indices: Vec<U32Id<BVoxValuePoolValue>>,
}

/// A four-component sRGB color pool over the extracted color, deduplicated by its
/// 8-bit bytes and stored as float components in `[0, 1]`.
fn srgba_pool(
    materials: &[MeshMaterial],
    get: impl Fn(&MeshMaterial) -> TySrgbaU8,
) -> PoolColumn<[f64; 4]> {
    let mut values = Vec::new();
    let mut lookup: HashMap<[u8; 4], U32Id<BVoxValuePoolValue>> = HashMap::new();
    let indices = materials
        .iter()
        .map(|material| {
            let color = get(material);
            *lookup.entry(<[u8; 4]>::from(color)).or_insert_with(|| {
                let index = U32Id::from_u32(values.len() as u32);
                values.push(<[f64; 4]>::from(color.into_format::<f64, f64>()));
                index
            })
        })
        .collect();
    PoolColumn { values, indices }
}

/// A three-component sRGB color pool over the extracted color, deduplicated by
/// its 8-bit bytes and stored as float components in `[0, 1]`; the alpha is
/// dropped.
fn srgb_pool(
    materials: &[MeshMaterial],
    get: impl Fn(&MeshMaterial) -> TySrgbaU8,
) -> PoolColumn<[f64; 3]> {
    let mut values = Vec::new();
    let mut lookup: HashMap<[u8; 3], U32Id<BVoxValuePoolValue>> = HashMap::new();
    let indices = materials
        .iter()
        .map(|material| {
            let color = get(material);
            *lookup
                .entry([color.red, color.green, color.blue])
                .or_insert_with(|| {
                    let index = U32Id::from_u32(values.len() as u32);
                    values.push(<[f64; 3]>::from(color.into_format::<f64, f64>().color));
                    index
                })
        })
        .collect();
    PoolColumn { values, indices }
}

/// A float pool over the extracted scalar, deduplicated by its bit pattern.
fn float_pool(materials: &[MeshMaterial], get: impl Fn(&MeshMaterial) -> f64) -> PoolColumn<f64> {
    let mut values = Vec::new();
    let mut lookup: HashMap<u64, U32Id<BVoxValuePoolValue>> = HashMap::new();
    let indices = materials
        .iter()
        .map(|material| {
            let value = get(material);
            *lookup.entry(value.to_bits()).or_insert_with(|| {
                let index = U32Id::from_u32(values.len() as u32);
                values.push(value);
                index
            })
        })
        .collect();
    PoolColumn { values, indices }
}

/// A float pool bounded on both sides.
fn bounded_float(values: Vec<f64>, min: f64, max: f64) -> Result<VoxValuePool> {
    Ok(VoxValuePool::float(
        VoxBound::Number(min),
        VoxBound::Number(max),
        values,
    )?)
}

/// A float pool bounded below and unbounded above.
fn float_above(values: Vec<f64>, min: f64) -> Result<VoxValuePool> {
    Ok(VoxValuePool::float(
        VoxBound::Number(min),
        VoxBound::None,
        values,
    )?)
}

/// A hashable identity for a material: its two 8-bit colors and the bit patterns
/// of its scalar factors, so cells with the same material map to one palette
/// material.
type MaterialKey = ([u8; 4], u64, u64, [u8; 3], u64, u64, u64, u64);

/// The [`MaterialKey`] for a material.
fn material_key(material: &MeshMaterial) -> MaterialKey {
    let emissive = material.emissive_factor;
    (
        <[u8; 4]>::from(material.base_color),
        material.metallic.to_bits(),
        material.roughness.to_bits(),
        [emissive.red, emissive.green, emissive.blue],
        material.emissive_strength.to_bits(),
        material.occlusion.to_bits(),
        material.ior.to_bits(),
        material.transmission.to_bits(),
    )
}

/// For each filled cell, the index of its nearest surface cell, by a
/// six-connected multi-source flood from every surface cell through filled
/// cells. Surface cells map to themselves; an interior region a fill never
/// reaches stays `None`.
fn nearest_surface_cell(grid: &VoxelGrid, counts: TyVector3U32) -> Vec<Option<usize>> {
    let (nx, ny, nz) = (counts.x as usize, counts.y as usize, counts.z as usize);

    let mut source: Vec<Option<usize>> = vec![None; grid.filled.len()];

    let mut queue: VecDeque<usize> = VecDeque::new();

    for (cell, triangle) in grid.triangle.iter().enumerate() {
        if triangle.is_some() {
            source[cell] = Some(cell);
            queue.push_back(cell);
        }
    }

    while let Some(cell) = queue.pop_front() {
        let origin = source[cell];
        for_each_neighbor(cell, nx, ny, nz, |next| {
            if grid.filled[next] && source[next].is_none() {
                source[next] = origin;
                queue.push_back(next);
            }
        });
    }

    source
}

/// Visits the six-connected in-grid neighbors of a raster cell index.
fn for_each_neighbor(cell: usize, nx: usize, ny: usize, nz: usize, mut visit: impl FnMut(usize)) {
    let plane = ny * nz;

    let (x, rem) = (cell / plane, cell % plane);

    let (y, z) = (rem / nz, rem % nz);

    if x > 0 {
        visit(cell - plane);
    }

    if x + 1 < nx {
        visit(cell + plane);
    }

    if y > 0 {
        visit(cell - nz);
    }

    if y + 1 < ny {
        visit(cell + nz);
    }

    if z > 0 {
        visit(cell - 1);
    }

    if z + 1 < nz {
        visit(cell + 1);
    }
}

/// The fill color as a stored sRGB color, defaulting to opaque white for `none`.
fn fill_srgba(fill_color: Option<[u8; 4]>) -> TySrgbaU8 {
    match fill_color {
        Some([r, g, b, a]) => TySrgbaU8::new(r, g, b, a),
        None => TySrgbaU8::from(DEFAULT_FILL),
    }
}

/// The error for a grid past voxcore's dense-grid cell limit.
fn grid_too_large(counts: TyVector3U32) -> Error {
    Error::invalid(format!(
        "voxel grid {}x{}x{} exceeds the dense limit of {} cells",
        counts.x,
        counts.y,
        counts.z,
        VoxObject::MAX_GRID_CELLS
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        EMISSIVE_STRENGTH, FillMode, MaterialMode, Mesh, MeshMaterial, MeshTriangle,
        MeshTriangleUvs, SurfaceMode, voxelize_mesh,
    };
    use ty_math::{TySrgbaU8, TyVector3F64, TyVector3U32};
    use voxcore::{VoxMain, VoxValuePoolValueRef};

    /// A two-cell mesh over a `2x1x1` grid: one triangle inside each unit
    /// cell, the left tagged material `0` and the right material `1`.
    fn two_cell_mesh(materials: Vec<MeshMaterial>) -> Mesh {
        let triangle = |cell: f64, material: u32| MeshTriangle {
            points: [
                TyVector3F64::new(cell + 0.2, 0.2, 0.5),
                TyVector3F64::new(cell + 0.8, 0.2, 0.5),
                TyVector3F64::new(cell + 0.5, 0.8, 0.5),
            ],
            uvs: MeshTriangleUvs::default(),
            material,
        };

        Mesh {
            triangles: vec![triangle(0.0, 0), triangle(1.0, 1)],
            maps: vec![Default::default(); materials.len()],
            materials,
            textures: Vec::new(),
            name: None,
        }
    }

    /// Voxelizes the two-cell mesh per-primitive.
    fn voxelize(materials: Vec<MeshMaterial>) -> VoxMain {
        voxelize_mesh(
            &two_cell_mesh(materials),
            TyVector3U32::new(2, 1, 1),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::PerPrimitive,
            None,
            1.0,
            None,
            "mesh",
        )
        .unwrap()
    }

    /// The strength the given voxel's material samples from the
    /// `emissiveStrength` property.
    fn sampled_strength(state: &VoxMain, position: TyVector3U32) -> f64 {
        let (_, object) = state.iter_objects().next().unwrap();
        let (layer, palette_id) = object.iter_layers().next().unwrap();
        let palette = state.palette(palette_id).unwrap();
        let property = palette.property_id_by_name(EMISSIVE_STRENGTH).unwrap();
        let voxel = object.voxel_id(position).unwrap();
        let material = object.voxel_material(voxel, layer).unwrap();
        match state
            .material_value(palette_id, material, property)
            .and_then(|(pool, value_id)| pool.value(value_id))
        {
            Some(VoxValuePoolValueRef::Float(number)) => number,
            other => panic!("expected a float pool, got {other:?}"),
        }
    }

    #[test]
    fn a_shared_strength_repeats_one_value_pool_value() {
        let mut emissive = MeshMaterial::flat(TySrgbaU8::new(255, 0, 0, 255));
        emissive.emissive_strength = 2.0;

        let mut other = MeshMaterial::flat(TySrgbaU8::new(0, 0, 255, 255));
        other.emissive_strength = 2.0;

        let state = voxelize(vec![emissive, other]);

        // Two distinct materials share the strength, so both rows repeat the
        // deduplicated pool's one value.
        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.iter_materials().count(), 2);
        let property = palette.property_id_by_name(EMISSIVE_STRENGTH).unwrap();
        let value_pool_id = palette.property(property).unwrap().value_pool_id;
        assert_eq!(state.value_pool(value_pool_id).unwrap().values_len(), 1);
        assert_eq!(sampled_strength(&state, TyVector3U32::new(0, 0, 0)), 2.0);
        assert_eq!(sampled_strength(&state, TyVector3U32::new(1, 0, 0)), 2.0);
    }

    #[test]
    fn mixed_strengths_sample_per_material() {
        let mut dim = MeshMaterial::flat(TySrgbaU8::new(255, 0, 0, 255));
        dim.emissive_strength = 1.0;

        let mut bright = MeshMaterial::flat(TySrgbaU8::new(255, 0, 0, 255));
        bright.emissive_strength = 3.0;

        let state = voxelize(vec![dim, bright]);

        assert_eq!(sampled_strength(&state, TyVector3U32::new(0, 0, 0)), 1.0);
        assert_eq!(sampled_strength(&state, TyVector3U32::new(1, 0, 0)), 3.0);
    }

    #[test]
    fn a_flat_fill_carries_the_default_strength() {
        // Flat mode paints one material with the default strength.
        let state = voxelize_mesh(
            &two_cell_mesh(Vec::new()),
            TyVector3U32::new(2, 1, 1),
            SurfaceMode::TriangleCover,
            FillMode::Surface,
            MaterialMode::Flat,
            None,
            1.0,
            None,
            "mesh",
        )
        .unwrap();

        assert_eq!(sampled_strength(&state, TyVector3U32::new(0, 0, 0)), 0.0);
    }
}
