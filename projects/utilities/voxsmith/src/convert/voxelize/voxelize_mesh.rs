use crate::{
    Error, FillMode, MATERIAL_ATTRIBUTES, MaterialMode, Mesh, MeshMaterial, Result, VoxelGrid,
    sample_base_color, voxelize_triangles,
};
use branded_id::U32Id;
use std::collections::{HashMap, VecDeque};
use ty_math::{TySrgbaColor, TyTransformF64, TyVector3, TyVector3U32};
use voxcore::{BVoxPaletteCell, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette};

/// The color a body with no sampled surface falls back to when `fill_color` is
/// `None`: opaque white.
const DEFAULT_FILL: TySrgbaColor = TySrgbaColor {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
};

/// Voxelizes a [`Mesh`] into a [`VoxMain`] of one object placed by one root
/// node. Errors when the mesh has no triangle geometry, the grid exceeds
/// voxcore's dense-grid limit, or the assembled state fails validation.
///
/// # Arguments
/// * `mesh` - the mesh to rasterize, in Z-up world space.
/// * `counts` - grid resolution in voxels per axis, sized by the caller from the
///   mesh extent (see [`Mesh::extent`]).
/// * `fill_mode` - the fill geometry.
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

    let grid = voxelize_triangles(&mesh.triangles, counts, fill_mode == FillMode::Solid);

    let cell_materials = resolve_materials(mesh, &grid, counts, material_mode, fill_color);

    let (palette, samples, default_cell) = build_palette(&cell_materials);

    let mut state = VoxMain::default();
    let palette_id = state.add_palette(palette);

    // The object name: an explicit override, else the mesh's own name, else the
    // caller's fallback.
    let object_name = name
        .map(str::to_owned)
        .or(mesh.name.clone())
        .unwrap_or_else(|| fallback_name.to_owned());

    let mut object = VoxObject::new(object_name, counts).ok_or_else(|| grid_too_large(counts))?;

    object.add_palette_ref(palette_id, default_cell);

    for (index, sample) in samples.iter().enumerate() {
        if let Some(cell) = sample {
            let voxel = U32Id::from_u32(index as u32);
            object
                .retain_voxel(voxel, &[*cell])
                .expect("a grid index is a live voxel sampling one reference");
        }
    }

    let object_id = state.add_object(object);

    // One root node placing the object and carrying the real-world scale.
    let transform = TyTransformF64 {
        scale: TyVector3::new(node_scale, node_scale, node_scale),
        ..Default::default()
    };

    let node = VoxHierarchyNode {
        child_objects: vec![object_id],
        transform,
        ..Default::default()
    };

    let node_id = state.add_hierarchy_node(node);

    state.push_root_hierarchy_node(node_id);

    state.validate()?;

    Ok(state)
}

/// The material of every filled cell, per the color mode: `flat` paints one fill
/// color, `per-primitive`/`per-texel` read the covering material (its base color
/// sampled per-texel), and `auto` samples a textured mesh and reads factors
/// otherwise.
fn resolve_materials(
    mesh: &Mesh,
    grid: &VoxelGrid,
    counts: TyVector3U32,
    material_mode: MaterialMode,
    fill_color: Option<[u8; 4]>,
) -> Vec<Option<MeshMaterial>> {
    let sample = match material_mode {
        MaterialMode::Flat => return flat_cells(grid, fill_color),
        MaterialMode::PerPrimitive => false,
        MaterialMode::PerTexel => true,
        MaterialMode::Auto => mesh.is_textured(),
    };

    sampled_cells(mesh, grid, counts, sample, fill_color)
}

/// Every filled cell takes the one fill color (white when `none`).
fn flat_cells(grid: &VoxelGrid, fill_color: Option<[u8; 4]>) -> Vec<Option<MeshMaterial>> {
    let material = MeshMaterial::flat(fill_srgba(fill_color));
    grid.filled
        .iter()
        .map(|&filled| filled.then_some(material))
        .collect()
}

/// Each surface cell takes its covering material's finish, its base color
/// sampled from the texture when `sample` (over the cell's footprint, or the
/// flat factor for an untextured material). A `solid` interior takes the fill
/// color when given, else its nearest surface cell's material.
fn sampled_cells(
    mesh: &Mesh,
    grid: &VoxelGrid,
    counts: TyVector3U32,
    sample: bool,
    fill_color: Option<[u8; 4]>,
) -> Vec<Option<MeshMaterial>> {
    let sampled = sample.then(|| {
        sample_base_color(
            &mesh.triangles,
            &mesh.base_colors,
            &mesh.textures,
            grid,
            counts,
        )
    });

    let mut cell_materials: Vec<Option<MeshMaterial>> = vec![None; grid.filled.len()];

    for (cell, &covering) in grid.triangle.iter().enumerate() {
        let Some(covering) = covering else { continue };
        let material = mesh.materials[mesh.triangles[covering as usize].material as usize];
        let rgba = sampled
            .as_ref()
            .and_then(|colors| colors[cell])
            .unwrap_or(material.rgba);
        cell_materials[cell] = Some(MeshMaterial { rgba, ..material });
    }

    fill_interior(grid, counts, fill_color, &mut cell_materials);

    cell_materials
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
            let fill = MeshMaterial::flat(TySrgbaColor::new(r, g, b, a));
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
                        .unwrap_or_else(|| MeshMaterial::flat(DEFAULT_FILL));
                    cell_materials[cell] = Some(resolved);
                }
            }
        }
    }
}

/// Assembles a palette from a per-cell material list: near-identical materials
/// merge to one cell, and each filled cell samples the cell of its material. The
/// default cell is the first built, or a lone white cell for an all-empty grid,
/// so the palette is never empty.
fn build_palette(
    cell_materials: &[Option<MeshMaterial>],
) -> (
    VoxPalette,
    Vec<Option<U32Id<BVoxPaletteCell>>>,
    U32Id<BVoxPaletteCell>,
) {
    let mut palette = material_palette();

    let mut cells: HashMap<MaterialKey, U32Id<BVoxPaletteCell>> = HashMap::new();

    let samples: Vec<Option<U32Id<BVoxPaletteCell>>> = cell_materials
        .iter()
        .map(|&material| {
            material.map(|material| {
                *cells
                    .entry(material_key(&material))
                    .or_insert_with(|| add_material(&mut palette, material))
            })
        })
        .collect();

    let first_cell = palette.iter_cells().next();

    let default_cell = match first_cell {
        Some(cell) => cell,
        None => add_material(&mut palette, MeshMaterial::flat(DEFAULT_FILL)),
    };

    (palette, samples, default_cell)
}

/// A hashable identity for a material: its 8-bit color and the bit patterns of
/// its finish, so cells with the same stored row map to one palette cell.
type MaterialKey = (u8, u8, u8, u8, u64, u64, u64, u64);

/// The [`MaterialKey`] for a material.
fn material_key(material: &MeshMaterial) -> MaterialKey {
    let TySrgbaColor { r, g, b, a } = material.rgba;
    (
        r,
        g,
        b,
        a,
        material.metallic.to_bits(),
        material.roughness.to_bits(),
        material.emissive.to_bits(),
        material.occlusion.to_bits(),
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

/// An empty palette carrying the five PBR material attributes every mode writes.
fn material_palette() -> VoxPalette {
    let mut palette = VoxPalette::default();

    for attribute in MATERIAL_ATTRIBUTES {
        palette.add_attribute(attribute.to_owned());
    }

    palette
}

/// Adds `material` as one cell and returns its id.
fn add_material(palette: &mut VoxPalette, material: MeshMaterial) -> U32Id<BVoxPaletteCell> {
    palette
        .add_cell(material.cell_values())
        .expect("one value for each of the five material attributes")
}

/// The fill color as a stored sRGB color, defaulting to opaque white for `none`.
fn fill_srgba(fill_color: Option<[u8; 4]>) -> TySrgbaColor {
    match fill_color {
        Some([r, g, b, a]) => TySrgbaColor::new(r, g, b, a),
        None => DEFAULT_FILL,
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
