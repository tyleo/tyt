use crate::{
    Error, FillMode, MATERIAL_ATTRIBUTES, MaterialMode, Mesh, MeshMaterial, Result, VoxelGrid,
    voxelize_triangles,
};
use branded_id::U32Id;
use std::collections::VecDeque;
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
/// * `material_mode` - the color source. Per-texel and `auto` fall back to
///   per-primitive until the texel sampler lands.
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

    let (palette, samples, default_cell) = match material_mode {
        MaterialMode::Flat => flat_palette(&grid, fill_color),

        // Per-texel sampling is not built yet; it and `auto` fall back to the
        // per-primitive path, which reads each material's flat factors. The
        // texel sampler and texture-aware `auto` land with the per-texel work.
        MaterialMode::Auto | MaterialMode::PerPrimitive | MaterialMode::PerTexel => {
            per_primitive_palette(mesh, &grid, counts, fill_color)
        }
    };

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

/// Builds the one-cell flat palette and its per-cell samples: every filled voxel
/// takes the fill color (white when `none`), with default finish.
fn flat_palette(
    grid: &VoxelGrid,
    fill_color: Option<[u8; 4]>,
) -> (
    VoxPalette,
    Vec<Option<U32Id<BVoxPaletteCell>>>,
    U32Id<BVoxPaletteCell>,
) {
    let mut palette = material_palette();
    let cell = add_material(&mut palette, MeshMaterial::flat(fill_srgba(fill_color)));
    let samples = grid.filled.iter().map(|&f| f.then_some(cell)).collect();

    (palette, samples, cell)
}

/// Builds the per-primitive palette and its per-cell samples: one cell per mesh
/// material a surface voxel uses, plus a fill cell for a `solid` interior when a
/// `fill_color` is given. Without one, interior voxels instead adopt their
/// nearest surface voxel's material.
fn per_primitive_palette(
    mesh: &Mesh,
    grid: &VoxelGrid,
    counts: TyVector3U32,
    fill_color: Option<[u8; 4]>,
) -> (
    VoxPalette,
    Vec<Option<U32Id<BVoxPaletteCell>>>,
    U32Id<BVoxPaletteCell>,
) {
    let keys = resolve_keys(grid, counts, fill_color);

    let mut palette = material_palette();

    // A cell per distinct surface material used, in ascending slot order.
    let mut used: Vec<u32> = keys
        .iter()
        .filter_map(|key| match key {
            Some(Key::Surface(slot)) => Some(*slot),
            _ => None,
        })
        .collect();

    used.sort_unstable();

    used.dedup();

    let mut slot_cells: Vec<(u32, U32Id<BVoxPaletteCell>)> = Vec::with_capacity(used.len());
    for slot in used {
        let cell = add_material(&mut palette, mesh.materials[slot as usize]);
        slot_cells.push((slot, cell));
    }

    // A single fill cell, shared by every interior voxel that resolved to it.
    let fill_cell = keys
        .iter()
        .any(|key| matches!(key, Some(Key::Fill)))
        .then(|| add_material(&mut palette, MeshMaterial::flat(fill_srgba(fill_color))));

    // A valid default sample for the empty voxels the reference back-fills; a
    // mesh that rasterized to nothing gets a lone white cell so the palette is
    // never empty.
    let first_cell = palette.iter_cells().next();

    let default_cell = match first_cell {
        Some(cell) => cell,
        None => add_material(&mut palette, MeshMaterial::flat(DEFAULT_FILL)),
    };

    let cell_of = |slot: u32| {
        slot_cells
            .iter()
            .find(|(s, _)| *s == slot)
            .map(|(_, cell)| *cell)
            .expect("every used surface slot has a cell")
    };

    let samples = keys
        .iter()
        .map(|key| {
            key.map(|key| match key {
                Key::Surface(slot) => cell_of(slot),
                Key::Fill => fill_cell.expect("a fill cell exists when a Fill key does"),
            })
        })
        .collect();

    (palette, samples, default_cell)
}

/// The material each filled voxel samples: a surface voxel takes its own
/// material, while a `solid` body's interior takes the fill color when one is
/// given, else the nearest surface voxel's material. Empty cells are `None`.
fn resolve_keys(
    grid: &VoxelGrid,
    counts: TyVector3U32,
    fill_color: Option<[u8; 4]>,
) -> Vec<Option<Key>> {
    let has_interior = grid
        .filled
        .iter()
        .zip(&grid.material)
        .any(|(&filled, material)| filled && material.is_none());

    // The nearest-surface map is only needed to paint a sampled interior, so
    // skip the flood when a fill color already answers the interior.
    let nearest = (fill_color.is_none() && has_interior).then(|| nearest_surface(grid, counts));

    grid.filled
        .iter()
        .enumerate()
        .map(|(cell, &filled)| {
            if !filled {
                return None;
            }
            Some(match grid.material[cell] {
                Some(slot) => Key::Surface(slot),
                None => match fill_color {
                    Some(_) => Key::Fill,
                    None => match nearest.as_ref().and_then(|map| map[cell]) {
                        Some(slot) => Key::Surface(slot),
                        None => Key::Fill,
                    },
                },
            })
        })
        .collect()
}

/// For each filled cell, the material slot of its nearest surface cell, by a
/// six-connected multi-source flood from every surface cell through filled
/// cells. Surface cells keep their own slot; an interior region a fill never
/// reaches stays `None`.
fn nearest_surface(grid: &VoxelGrid, counts: TyVector3U32) -> Vec<Option<u32>> {
    let (nx, ny, nz) = (counts.x as usize, counts.y as usize, counts.z as usize);

    let mut slot = grid.material.clone();

    let mut queue: VecDeque<usize> = grid
        .material
        .iter()
        .enumerate()
        .filter_map(|(cell, material)| material.map(|_| cell))
        .collect();

    while let Some(cell) = queue.pop_front() {
        let source = slot[cell];
        for next in neighbors(cell, nx, ny, nz) {
            if grid.filled[next] && slot[next].is_none() {
                slot[next] = source;
                queue.push_back(next);
            }
        }
    }

    slot
}

/// The six-connected in-grid neighbors of a raster cell index.
fn neighbors(cell: usize, nx: usize, ny: usize, nz: usize) -> Vec<usize> {
    let plane = ny * nz;

    let (x, rem) = (cell / plane, cell % plane);

    let (y, z) = (rem / nz, rem % nz);

    let mut out = Vec::with_capacity(6);

    if x > 0 {
        out.push(cell - plane);
    }

    if x + 1 < nx {
        out.push(cell + plane);
    }

    if y > 0 {
        out.push(cell - nz);
    }

    if y + 1 < ny {
        out.push(cell + nz);
    }

    if z > 0 {
        out.push(cell - 1);
    }

    if z + 1 < nz {
        out.push(cell + 1);
    }

    out
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

/// One filled voxel's material choice.
#[derive(Clone, Copy)]
enum Key {
    /// The mesh material at this slot.
    Surface(u32),

    /// The shared fill-color cell.
    Fill,
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
