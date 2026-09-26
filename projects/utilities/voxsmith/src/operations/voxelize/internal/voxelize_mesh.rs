use crate::{
    Error, Result,
    operations::voxelize::{
        FillMode, GridSpace, MaterialMode, MeshInput, OutOfRangeProperty, SurfaceMode, VoxelGrid,
        VoxelMaterial, VoxelizeOptions, sample_material, voxelize_triangles,
    },
    utilities::{check_material_property_ranges, check_material_range},
};
use branded_id::U32Id;
use meshdoc::material::{COLOR_RANGE, MaterialRange, scalar_range};
use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64, TySrgbaU8, TyTransformF64, TyVector3U32};
use voxcore::{
    BVoxMaterial, BVoxValuePoolValue, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
    VoxValuePool,
    color::lin_srgba_f64_from_srgba_u8,
    material::{
        BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, OCCLUSION_STRENGTH,
        ROUGHNESS, TRANSMISSION,
    },
};

/// The color a body with no sampled surface falls back to when `fill_color` is
/// `None`: opaque white. Held as sRGB bytes, the form a caller's fill color
/// arrives in. Decode it with [`lin_srgba_f64_from_srgba_u8`] at each use site.
const DEFAULT_FILL: [u8; 4] = [255, 255, 255, 255];

/// Voxelizes a [`MeshInput`] onto `space` into a [`VoxMain`] of one object
/// placed by one root node, whose scale records the voxel size. Errors when
/// the grid exceeds voxcore's dense-grid limit.
///
/// # Arguments
/// * `mesh` - the mesh to rasterize, in world space.
/// * `space` - the grid to rasterize onto.
/// * `fallback_name` - the object name when neither `options.name` nor the
///   mesh has one.
/// * `options` - everything but the resolution, which the caller resolved
///   into `space`.
pub fn voxelize_mesh(
    mesh: &MeshInput<'_>,
    space: &GridSpace,
    fallback_name: &str,
    options: &VoxelizeOptions,
) -> Result<VoxMain> {
    let counts = space.counts();

    // Cap the grid before rasterizing. An oversized resolution errors before
    // the occupancy grid allocation overflows or exhausts memory.
    let volume = counts.x as u64 * counts.y as u64 * counts.z as u64;
    if volume > VoxObject::MAX_GRID_CELLS {
        return Err(grid_too_large(counts));
    }

    let grid = voxelize_triangles(
        &mesh.triangles,
        space,
        options.surface_mode == SurfaceMode::CenterInside,
        options.fill_mode == FillMode::Solid,
    );

    let cell_materials = resolve_materials(
        mesh,
        &grid,
        space,
        options.material_mode,
        options.fill_color,
    );

    let mut main = VoxMain::default();

    let (palette, sample_ids, default_material_id) =
        build_palette(&mut main, &cell_materials, options.out_of_range_property)?;

    let palette_id = main.retain_palette(palette)?;

    let object_name = options
        .name
        .clone()
        .or(mesh.name.clone())
        .unwrap_or_else(|| fallback_name.to_owned());

    let mut object = VoxObject::new(object_name, counts).map_err(|_| grid_too_large(counts))?;

    object.retain_layer(palette_id, default_material_id);

    for (index, sample_id) in sample_ids.iter().enumerate() {
        if let Some(material_id) = sample_id {
            let voxel_id = U32Id::from_u32(index as u32);
            object
                .retain_voxel(voxel_id, &[*material_id])
                .expect("a grid index is a live voxel sampling the one layer");
        }
    }

    let object_id = main.retain_object(object)?;

    let transform = TyTransformF64 {
        scale: space.size(),
        ..Default::default()
    };

    let node = VoxHierarchyNode {
        child_object_ids: vec![object_id],
        transform,
        ..Default::default()
    };

    let node_id = main.retain_hierarchy_node(node)?;

    main.push_root_hierarchy_node_id(node_id)?;

    // The values were checked or clamped above. This run guarantees nothing
    // out of range leaves the import.
    check_material_property_ranges(&main)?;

    Ok(main)
}

/// The material of every filled cell under `material_mode`.
fn resolve_materials(
    mesh: &MeshInput<'_>,
    grid: &VoxelGrid,
    space: &GridSpace,
    material_mode: MaterialMode,
    fill_color: Option<[u8; 4]>,
) -> Vec<Option<VoxelMaterial>> {
    let mut cell_materials = match material_mode {
        MaterialMode::Flat => return flat_cells(grid, fill_color),

        MaterialMode::PerPrimitive => primitive_cells(mesh, grid),

        MaterialMode::PerTexel => sample_material(mesh, grid, space),

        MaterialMode::Auto if mesh.is_textured() => sample_material(mesh, grid, space),

        MaterialMode::Auto => primitive_cells(mesh, grid),
    };

    fill_interior(grid, space.counts(), fill_color, &mut cell_materials);

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
fn flat_cells(grid: &VoxelGrid, fill_color: Option<[u8; 4]>) -> Vec<Option<VoxelMaterial>> {
    let material = VoxelMaterial::flat(fill_lin_srgba_f64_color(fill_color));
    grid.filled
        .iter()
        .map(|&filled| filled.then_some(material))
        .collect()
}

/// Each surface cell takes its covering triangle's flat material; interior and
/// empty cells are `None`.
fn primitive_cells(mesh: &MeshInput<'_>, grid: &VoxelGrid) -> Vec<Option<VoxelMaterial>> {
    let materials: Vec<VoxelMaterial> = mesh
        .primitives
        .iter()
        .map(|placed| VoxelMaterial::from(placed.material))
        .collect();

    grid.triangle
        .iter()
        .map(|&covering| {
            covering.map(|triangle| materials[mesh.triangles[triangle as usize].primitive as usize])
        })
        .collect()
}

/// Paints every filled interior cell, the volume a `solid` fill invents with
/// no surface material: the fill color when given, else its nearest surface
/// cell's material. A per-texel sample can leave that surface cell without a
/// material; white stands in there.
fn fill_interior(
    grid: &VoxelGrid,
    counts: TyVector3U32,
    fill_color: Option<[u8; 4]>,
    cell_materials: &mut [Option<VoxelMaterial>],
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
        Some(color) => {
            let fill = VoxelMaterial::flat(fill_lin_srgba_f64_color(Some(color)));
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
                        .unwrap_or_else(|| VoxelMaterial::flat(fill_lin_srgba_f64_color(None)));
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

/// Assembles a palette from a per-cell material list. Identical materials
/// merge to one palette material. Every vocabulary property draws from a
/// deduplicated value pool added to `main`. The default material is the
/// first built, or a lone white material for an all-empty grid, so the
/// palette is never empty. A value outside its property's range follows
/// `out_of_range`.
fn build_palette(
    main: &mut VoxMain,
    cell_materials: &[Option<VoxelMaterial>],
    out_of_range: OutOfRangeProperty,
) -> Result<PaletteBuild> {
    // Merge identical materials into a distinct list, first seen in raster
    // order, remembering each filled cell's position in it.
    let mut distinct: Vec<VoxelMaterial> = Vec::new();
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

    // An all-empty grid still needs a non-empty palette so its value pools and
    // default material are valid; give it a lone white material.
    if distinct.is_empty() {
        distinct.push(VoxelMaterial::flat(fill_lin_srgba_f64_color(None)));
    }

    // The color properties follow the same policy as the scalars: every
    // component of `baseColor`, alpha included, and `emissiveColor` lies in
    // `[0, 1]`.
    for material in &mut distinct {
        material.base_color = TyLinSrgbaF64::from(property_color(
            <[f64; 4]>::from(material.base_color),
            BASE_COLOR,
            out_of_range,
        )?);
        material.emissive_color = TyLinSrgbF64::from(property_color(
            <[f64; 3]>::from(material.emissive_color),
            EMISSIVE_COLOR,
            out_of_range,
        )?);
    }

    // One deduplicated value pool per property, plus each distinct material's
    // value id into it.
    let base_color = lin_srgba_f64_value_pool(&distinct, |material| material.base_color);
    let metallic = f64_value_pool(&distinct, |material| material.metallic);
    let roughness = f64_value_pool(&distinct, |material| material.roughness);
    let emissive_color = lin_srgb_f64_value_pool(&distinct, |material| material.emissive_color);
    let emissive_strength = f64_value_pool(&distinct, |material| material.emissive_strength);
    let occlusion = f64_value_pool(&distinct, |material| material.occlusion);
    let ior = f64_value_pool(&distinct, |material| material.ior);
    let transmission = f64_value_pool(&distinct, |material| material.transmission);

    // Register the value pools and add each property. All properties precede
    // any material, so no material carries a back-fill placeholder value id.
    let scalars = |values, key| property_value_pool(values, key, out_of_range);
    let base_color_value_pool_id =
        main.retain_value_pool(VoxValuePool::vec_4_float(base_color.values)?);
    let metallic_value_pool_id = main.retain_value_pool(scalars(metallic.values, METALLIC)?);
    let roughness_value_pool_id = main.retain_value_pool(scalars(roughness.values, ROUGHNESS)?);
    let emissive_color_value_pool_id =
        main.retain_value_pool(VoxValuePool::vec_3_float(emissive_color.values)?);
    let emissive_strength_value_pool_id =
        main.retain_value_pool(scalars(emissive_strength.values, EMISSIVE_STRENGTH)?);
    let occlusion_value_pool_id =
        main.retain_value_pool(scalars(occlusion.values, OCCLUSION_STRENGTH)?);
    let ior_value_pool_id = main.retain_value_pool(scalars(ior.values, IOR)?);
    let transmission_value_pool_id =
        main.retain_value_pool(scalars(transmission.values, TRANSMISSION)?);

    let mut palette = VoxPalette::default();
    palette
        .retain_property(
            BASE_COLOR.to_owned(),
            base_color_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .retain_property(
            METALLIC.to_owned(),
            metallic_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .retain_property(
            ROUGHNESS.to_owned(),
            roughness_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .retain_property(
            EMISSIVE_COLOR.to_owned(),
            emissive_color_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .retain_property(
            EMISSIVE_STRENGTH.to_owned(),
            emissive_strength_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .retain_property(
            OCCLUSION_STRENGTH.to_owned(),
            occlusion_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    palette
        .retain_property(IOR.to_owned(), ior_value_pool_id, U32Id::from_u32(0))
        .expect("the property names are distinct");
    palette
        .retain_property(
            TRANSMISSION.to_owned(),
            transmission_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");

    // One material per distinct mesh material, its value ids in property
    // order.
    let material_ids: Vec<U32Id<BVoxMaterial>> = (0..distinct.len())
        .map(|index| {
            palette
                .retain_material(vec![
                    base_color.value_ids[index],
                    metallic.value_ids[index],
                    roughness.value_ids[index],
                    emissive_color.value_ids[index],
                    emissive_strength.value_ids[index],
                    occlusion.value_ids[index],
                    ior.value_ids[index],
                    transmission.value_ids[index],
                ])
                .expect("one value id for each property")
        })
        .collect();

    let sample_ids = cell_indices
        .iter()
        .map(|&index| index.map(|index| material_ids[index]))
        .collect();

    let default_material_id = material_ids[0];

    Ok((palette, sample_ids, default_material_id))
}

/// A deduplicated value pool column and each distinct material's value id into
/// it.
struct ValuePoolColumn<T> {
    /// The distinct values, in first-seen order.
    values: Vec<T>,

    /// Per distinct material, its value id into [`values`](Self::values).
    value_ids: Vec<U32Id<BVoxValuePoolValue>>,
}

/// A four-component color value pool over the extracted linear color,
/// deduplicated by its components' bit patterns.
fn lin_srgba_f64_value_pool(
    materials: &[VoxelMaterial],
    get: impl Fn(&VoxelMaterial) -> TyLinSrgbaF64,
) -> ValuePoolColumn<[f64; 4]> {
    value_pool_column(
        materials,
        |material| <[f64; 4]>::from(get(material)),
        |color| color.map(f64::to_bits),
    )
}

/// A three-component color value pool over the extracted linear color,
/// deduplicated by its components' bit patterns.
fn lin_srgb_f64_value_pool(
    materials: &[VoxelMaterial],
    get: impl Fn(&VoxelMaterial) -> TyLinSrgbF64,
) -> ValuePoolColumn<[f64; 3]> {
    value_pool_column(
        materials,
        |material| <[f64; 3]>::from(get(material)),
        |color| color.map(f64::to_bits),
    )
}

/// A float value pool over the extracted scalar, deduplicated by its bit
/// pattern.
fn f64_value_pool(
    materials: &[VoxelMaterial],
    get: impl Fn(&VoxelMaterial) -> f64,
) -> ValuePoolColumn<f64> {
    value_pool_column(materials, |material| get(material), |value| value.to_bits())
}

/// A deduplicated value pool column: each material's extracted value interned
/// by `key`, the distinct values kept in first-seen order.
fn value_pool_column<T, K: Eq + Hash>(
    materials: &[VoxelMaterial],
    get: impl Fn(&VoxelMaterial) -> T,
    key: impl Fn(&T) -> K,
) -> ValuePoolColumn<T> {
    let mut values = Vec::new();
    let mut lookup: HashMap<K, U32Id<BVoxValuePoolValue>> = HashMap::new();
    let value_ids = materials
        .iter()
        .map(|material| {
            let value = get(material);
            *lookup.entry(key(&value)).or_insert_with(|| {
                let value_id = U32Id::from_u32(values.len() as u32);
                values.push(value);
                value_id
            })
        })
        .collect();
    ValuePoolColumn { values, value_ids }
}

/// A float value pool over `values`, checked against the range the vocabulary
/// gives the property `key`. A value outside that range follows
/// `out_of_range`.
fn property_value_pool(
    values: Vec<f64>,
    key: &str,
    out_of_range: OutOfRangeProperty,
) -> Result<VoxValuePool> {
    let range = scalar_range(key).expect("the voxelizer writes vocabulary scalar attributes");

    let values = values
        .into_iter()
        .map(|value| property_value(value, range, key, out_of_range))
        .collect::<Result<Vec<f64>>>()?;

    Ok(VoxValuePool::float(values)?)
}

/// A color's components, each checked against [`COLOR_RANGE`]. A component
/// outside it follows `out_of_range`.
fn property_color<const N: usize>(
    mut components: [f64; N],
    key: &str,
    out_of_range: OutOfRangeProperty,
) -> Result<[f64; N]> {
    for component in &mut components {
        *component = property_value(*component, COLOR_RANGE, key, out_of_range)?;
    }
    Ok(components)
}

/// One property value under the out-of-range policy: itself when it lies in
/// `range`, its clamp under [`OutOfRangeProperty::Clamp`], else the range
/// error. An in-range value is its own clamp, which keeps the clamp off
/// `ior`'s admitted zero. A non-finite value has no clamp on the interval
/// and errors under either policy.
fn property_value(
    value: f64,
    range: MaterialRange,
    key: &str,
    out_of_range: OutOfRangeProperty,
) -> Result<f64> {
    if range.contains(value) {
        return Ok(value);
    }
    if out_of_range == OutOfRangeProperty::Clamp && value.is_finite() {
        return Ok(range.clamp(value));
    }
    check_material_range(key, value, range)?;
    Ok(value)
}

/// A hashable identity for a material: the bit patterns of its two colors'
/// components and its scalar factors, so cells with the same material map to
/// one palette material.
type MaterialKey = ([u64; 4], u64, u64, [u64; 3], u64, u64, u64, u64);

/// The [`MaterialKey`] for a material.
fn material_key(material: &VoxelMaterial) -> MaterialKey {
    (
        <[f64; 4]>::from(material.base_color).map(f64::to_bits),
        material.metallic.to_bits(),
        material.roughness.to_bits(),
        <[f64; 3]>::from(material.emissive_color).map(f64::to_bits),
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

/// The fill color decoded to linear light, defaulting to opaque white for
/// `none`.
fn fill_lin_srgba_f64_color(fill_color: Option<[u8; 4]>) -> TyLinSrgbaF64 {
    lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(fill_color.unwrap_or(DEFAULT_FILL)))
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
    use super::build_palette;
    use crate::{
        Result,
        operations::voxelize::{OutOfRangeProperty, VoxelMaterial},
    };
    use branded_id::U32Id;
    use ty_math::TyLinSrgbaF64;
    use voxcore::{
        BVoxMaterial, BVoxPalette, VoxMain, VoxValuePoolValueRef,
        material::{BASE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC},
    };

    /// A palette over one filled cell per material, retained into a main,
    /// with each cell's material id.
    fn palette_of(
        materials: Vec<VoxelMaterial>,
        out_of_range: OutOfRangeProperty,
    ) -> Result<(VoxMain, U32Id<BVoxPalette>, Vec<U32Id<BVoxMaterial>>)> {
        let mut main = VoxMain::default();
        let cells: Vec<Option<VoxelMaterial>> = materials.into_iter().map(Some).collect();

        let (palette, sample_ids, _) = build_palette(&mut main, &cells, out_of_range)?;

        let palette_id = main.retain_palette(palette)?;
        let material_ids = sample_ids
            .into_iter()
            .map(|sample_id| sample_id.expect("every cell is filled"))
            .collect();
        Ok((main, palette_id, material_ids))
    }

    /// The float `property` holds for `material_id`.
    fn number(
        main: &VoxMain,
        palette_id: U32Id<BVoxPalette>,
        material_id: U32Id<BVoxMaterial>,
        property: &str,
    ) -> f64 {
        let palette = main.palette(palette_id).unwrap();
        let property_id = palette.property_id_by_name(property).unwrap();
        match main
            .material_value(palette_id, material_id, property_id)
            .and_then(|(value_pool, value_id)| value_pool.value(value_id))
        {
            Some(VoxValuePoolValueRef::Float(number)) => number,
            other => panic!("expected a float value pool, got {other:?}"),
        }
    }

    /// The values of the pool `property` draws from, in listing order.
    fn values<'a>(
        main: &'a VoxMain,
        palette_id: U32Id<BVoxPalette>,
        property: &str,
    ) -> Vec<VoxValuePoolValueRef<'a>> {
        let palette = main.palette(palette_id).unwrap();
        let property_id = palette.property_id_by_name(property).unwrap();
        let value_pool_id = palette.property(property_id).unwrap().value_pool_id;
        let value_pool = main.value_pool(value_pool_id).unwrap();
        value_pool
            .iter_values()
            .map(|(value_id, _)| value_pool.value(value_id).unwrap())
            .collect()
    }

    /// A flat red material.
    fn red() -> VoxelMaterial {
        VoxelMaterial::flat(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0))
    }

    /// A flat blue material.
    fn blue() -> VoxelMaterial {
        VoxelMaterial::flat(TyLinSrgbaF64::new(0.0, 0.0, 1.0, 1.0))
    }

    /// `material` with `emissive_strength`.
    fn glowing(mut material: VoxelMaterial, emissive_strength: f64) -> VoxelMaterial {
        material.emissive_strength = emissive_strength;
        material
    }

    #[test]
    fn a_shared_strength_repeats_one_value_pool_value() {
        let (main, palette_id, material_ids) = palette_of(
            vec![glowing(red(), 2.0), glowing(blue(), 2.0)],
            OutOfRangeProperty::Error,
        )
        .unwrap();

        // Two distinct materials share the strength, so both rows repeat the
        // deduplicated value pool's one value.
        assert_eq!(main.palette(palette_id).unwrap().material_count(), 2);
        assert_eq!(values(&main, palette_id, EMISSIVE_STRENGTH).len(), 1);
        assert_eq!(
            number(&main, palette_id, material_ids[0], EMISSIVE_STRENGTH),
            2.0
        );
        assert_eq!(
            number(&main, palette_id, material_ids[1], EMISSIVE_STRENGTH),
            2.0
        );
    }

    #[test]
    fn identical_materials_merge_and_mixed_strengths_sample_per_material() {
        let (main, palette_id, material_ids) = palette_of(
            vec![
                glowing(red(), 1.0),
                glowing(red(), 3.0),
                glowing(red(), 1.0),
            ],
            OutOfRangeProperty::Error,
        )
        .unwrap();

        assert_eq!(main.palette(palette_id).unwrap().material_count(), 2);
        assert_eq!(material_ids[0], material_ids[2]);
        assert_eq!(
            number(&main, palette_id, material_ids[0], EMISSIVE_STRENGTH),
            1.0
        );
        assert_eq!(
            number(&main, palette_id, material_ids[1], EMISSIVE_STRENGTH),
            3.0
        );
    }

    #[test]
    fn an_empty_grid_gets_a_lone_white_material() {
        let mut main = VoxMain::default();
        let (palette, sample_ids, default_material_id) =
            build_palette(&mut main, &[None, None], OutOfRangeProperty::Error).unwrap();

        assert_eq!(palette.material_count(), 1);
        assert_eq!(sample_ids, [None, None]);
        assert_eq!(default_material_id, U32Id::from_u32(0));
    }

    /// Materials with the second one's `metallic` out of its range.
    fn over_metallic() -> Vec<VoxelMaterial> {
        let mut over = blue();
        over.metallic = 1.5;

        vec![red(), over]
    }

    #[test]
    fn an_out_of_range_scalar_errors_by_default() {
        let error = palette_of(over_metallic(), OutOfRangeProperty::Error).unwrap_err();

        // The property and the value both point back at the source mesh.
        let message = error.to_string();
        assert!(message.contains(METALLIC), "{message}");
        assert!(message.contains("1.5"), "{message}");
    }

    #[test]
    fn an_out_of_range_scalar_clamps_when_asked() {
        let (main, palette_id, _) = palette_of(over_metallic(), OutOfRangeProperty::Clamp).unwrap();

        // Both materials land on the value pool, the out-of-range one at the
        // range's top.
        assert_eq!(
            values(&main, palette_id, METALLIC),
            vec![
                VoxValuePoolValueRef::Float(0.0),
                VoxValuePoolValueRef::Float(1.0)
            ]
        );
    }

    #[test]
    fn an_ior_of_zero_passes_the_union_range() {
        // `ior` admits exactly 0 for "does not refract" alongside 1 and up.
        let mut hollow = blue();
        hollow.ior = 0.0;

        assert!(palette_of(vec![red(), hollow], OutOfRangeProperty::Error).is_ok());
    }

    /// Materials with the second one's `baseColor` red outside `[0, 1]`.
    fn over_red() -> Vec<VoxelMaterial> {
        let hot = VoxelMaterial::flat(TyLinSrgbaF64::new(2.0, 0.0, 0.0, 1.0));
        vec![red(), hot]
    }

    #[test]
    fn an_out_of_range_color_errors_by_default() {
        let error = palette_of(over_red(), OutOfRangeProperty::Error).unwrap_err();

        let message = error.to_string();
        assert!(message.contains("baseColor"), "{message}");
        assert!(message.contains('2'), "{message}");
    }

    #[test]
    fn an_out_of_range_color_clamps_when_asked() {
        let (main, palette_id, _) = palette_of(over_red(), OutOfRangeProperty::Clamp).unwrap();

        // The hot red clamps onto the matte one, so the deduplicated
        // baseColor value pool holds one value.
        assert_eq!(
            values(&main, palette_id, BASE_COLOR),
            vec![VoxValuePoolValueRef::Vec4Float(&[1.0, 0.0, 0.0, 1.0])]
        );
    }

    #[test]
    fn a_nan_scalar_errors_under_either_mode() {
        // A NaN has no clamped value, so asking to clamp does not admit it.
        let nan_metallic = || {
            let mut broken = blue();
            broken.metallic = f64::NAN;
            vec![red(), broken]
        };

        assert!(palette_of(nan_metallic(), OutOfRangeProperty::Error).is_err());
        assert!(palette_of(nan_metallic(), OutOfRangeProperty::Clamp).is_err());
    }

    #[test]
    fn an_infinite_scalar_errors_under_either_mode() {
        // `emissiveStrength` is unbounded above, but an unbounded top means
        // arbitrarily large and finite. An infinity has no clamp on the
        // interval either.
        let infinite_strength = || vec![red(), glowing(blue(), f64::INFINITY)];

        assert!(palette_of(infinite_strength(), OutOfRangeProperty::Error).is_err());
        assert!(palette_of(infinite_strength(), OutOfRangeProperty::Clamp).is_err());
    }

    #[test]
    fn the_clamp_leaves_the_ior_union_zero_alone() {
        // `ior` admits `{0} union [1, inf)`. Zero is in range, so the clamp
        // leaves it alone. Only a value between the union's parts lands on
        // the interval's end.
        let mut refracting = red();
        refracting.ior = 0.0;
        let mut between = blue();
        between.ior = 0.5;

        let (main, palette_id, _) =
            palette_of(vec![refracting, between], OutOfRangeProperty::Clamp).unwrap();
        let values = values(&main, palette_id, IOR);

        assert!(
            values.contains(&VoxValuePoolValueRef::Float(0.0)),
            "the admitted zero clamped away: {values:?}"
        );
        assert!(
            values.contains(&VoxValuePoolValueRef::Float(1.0)),
            "{values:?}"
        );
    }
}
