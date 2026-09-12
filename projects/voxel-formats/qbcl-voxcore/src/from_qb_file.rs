use crate::{Error, QbVoxMain, Result, qb_ext_from_file};
use branded_id::U32Id;
use qbcl::qb::{QbFile, QbMatrix};
use std::collections::{HashMap, HashSet};
use ty_math::{TySrgbaU8, TyTransformF64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxMaterial, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
    color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
};

/// Loads a decoded Qubicle Binary [`QbFile`] into a [`QbVoxMain`], the
/// inverse of [`to_qb_file`](crate::to_qb_file). Each matrix becomes an
/// object sharing one `baseColor` palette, placed by a root hierarchy node
/// named for the matrix at the matrix's scene position. The header flags go
/// to the ext. The per-voxel visibility bytes are not kept: the writer
/// derives them from the grid.
///
/// Errors on a matrix grid that exceeds the dense limit, or on a
/// cross-reference the checked insertions reject.
pub fn from_qb_file(file: &QbFile) -> Result<QbVoxMain> {
    let mut main = VoxMain::default();

    let (palette, material_ids) = build_palette(&mut main, file);
    let palette_id = main.retain_palette(palette)?;

    let mut root_ids = Vec::with_capacity(file.matrices.len());
    for matrix in &file.matrices {
        // The matrix grid becomes the object's build volume directly. It may
        // carry empty margin around the live voxels.
        let object = build_object(matrix, palette_id, &material_ids)?;
        let object_id = main.retain_object(object)?;
        let node = VoxHierarchyNode {
            name: matrix.name.clone(),
            child_node_ids: Vec::new(),
            child_object_ids: vec![object_id],
            transform: translation(matrix.position),
        };
        root_ids.push(main.retain_hierarchy_node(node)?);
    }
    main.set_root_hierarchy_node_ids(root_ids)?;

    Ok(main.put_ext(qb_ext_from_file(file)))
}

/// Builds the one shared palette: a color value pool of one entry per distinct
/// color across every matrix's solid voxels, bound to `baseColor`, with
/// one material per color and a map from a color to its material. The
/// value pool is added to `main`. A file with no solid voxels gets a single
/// placeholder color so objects have a default material to sample.
fn build_palette(
    main: &mut VoxMain<()>,
    file: &QbFile,
) -> (VoxPalette, HashMap<[u8; 3], U32Id<BVoxMaterial>>) {
    let mut order: Vec<[u8; 3]> = Vec::new();
    let mut seen: HashSet<[u8; 3]> = HashSet::new();
    for matrix in &file.matrices {
        for voxel in &matrix.voxels {
            if voxel.is_empty() {
                continue;
            }
            let color = [voxel.r, voxel.g, voxel.b];
            if seen.insert(color) {
                order.push(color);
            }
        }
    }
    if order.is_empty() {
        order.push([0, 0, 0]);
    }

    // A Qubicle voxel carries no alpha, so colors decode to linear light and
    // ride in a shared `vec-3-float` value pool. Each material draws one value
    // id into it.
    let value_pool_id = main.retain_value_pool(
        VoxValuePool::vec_3_float(order.iter().map(|&color| color_floats(color)).collect())
            .expect("byte-derived components are finite and the list is non-empty"),
    );

    let mut palette = VoxPalette::default();
    palette
        .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
        .expect("the property names are distinct");
    let mut material_ids = HashMap::with_capacity(order.len());
    for (index, color) in order.iter().enumerate() {
        let material_id = palette
            .retain_material(vec![U32Id::from_u32(index as u32)])
            .expect("one value id for the one property");
        material_ids.insert(*color, material_id);
    }

    (palette, material_ids)
}

/// Builds an object from a matrix: a dense grid sized by the matrix,
/// referencing the shared palette on one layer, each solid voxel sampling its
/// color material. Errors on an oversized grid.
fn build_object(
    matrix: &QbMatrix,
    palette_id: U32Id<BVoxPalette>,
    material_ids: &HashMap<[u8; 3], U32Id<BVoxMaterial>>,
) -> Result<VoxObject> {
    let [size_x, size_y, size_z] = matrix.size;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size_x, size_y, size_z))
        .map_err(|_| {
            Error::invalid(format!(
                "matrix grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));

    for z in 0..size_z {
        for y in 0..size_y {
            for x in 0..size_x {
                let Some(voxel) = matrix.voxel(x, y, z) else {
                    continue;
                };
                if voxel.is_empty() {
                    continue;
                }
                let material_id = material_ids
                    .get(&[voxel.r, voxel.g, voxel.b])
                    .copied()
                    .expect("every solid color is in the palette");
                let voxel_id = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .expect("a coordinate inside the matrix is inside the grid");
                object
                    .retain_voxel(voxel_id, &[material_id])
                    .expect("one sample for the one layer");
            }
        }
    }

    Ok(object)
}

/// A translation-only transform from a scene position.
fn translation(position: [i32; 3]) -> TyTransformF64 {
    TyTransformF64::from_translation(TyVector3I32::from_array(position).as_dvec3())
}

/// The linear-light components of an `[r, g, b]` byte color.
fn color_floats(color: [u8; 3]) -> [f64; 3] {
    let [red, green, blue] = color;
    let linear = lin_srgba_f64_from_srgba_u8(TySrgbaU8::new(red, green, blue, 255));
    [linear.red, linear.green, linear.blue]
}

#[cfg(test)]
mod tests {
    use crate::from_qb_file;
    use qbcl::qb::{QbFile, QbMatrix};

    #[test]
    fn rejects_an_oversized_matrix() {
        let file = QbFile {
            matrices: vec![QbMatrix {
                name: String::new(),
                size: [2048, 2048, 2048],
                position: [0, 0, 0],
                voxels: Vec::new(),
            }],
            ..Default::default()
        };
        assert!(from_qb_file(&file).is_err());
    }
}
