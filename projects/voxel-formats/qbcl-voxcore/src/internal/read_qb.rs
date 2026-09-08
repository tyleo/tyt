use crate::{
    Error, Result,
    ext::{QbExt, QbExtMatrix},
};
use branded_id::U32Id;
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbZAxisOrientation};
use std::collections::{HashMap, HashSet};
use ty_math::{TySrgbaU8, TyTransformF64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxMaterial, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
    color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
};

/// Loads a decoded Qubicle Binary [`QbFile`] into a bare [`VoxMain`] and the
/// [`QbExt`] that writes it back exactly. Each matrix becomes an object
/// sharing one `baseColor` palette, placed by a hierarchy node at the
/// matrix's scene position. The rest of the Qubicle state, such as the header
/// flags, matrix names, positions, and the per-voxel visibility bytes, goes
/// to the ext.
///
/// Errors on a matrix grid that exceeds the dense limit, or on a
/// cross-reference the checked insertions reject.
pub fn read_qb(file: &QbFile) -> Result<(VoxMain<()>, QbExt)> {
    let mut state = VoxMain::default();

    let (palette, material_ids) = build_palette(&mut state, file);
    let palette_id = state.retain_palette(palette)?;

    let mut matrices = Vec::with_capacity(file.matrices.len());
    let mut root_ids = Vec::with_capacity(file.matrices.len());
    for matrix in &file.matrices {
        // The matrix grid becomes the object's build volume directly; it may
        // carry empty margin around the live voxels. The visibility bytes are
        // read from that same grid.
        let object = build_object(matrix, palette_id, &material_ids)?;
        let visibility = visibility_of(&object, matrix);
        let object_id = state.retain_object(object)?;
        let node = VoxHierarchyNode {
            name: matrix.name.clone(),
            child_node_ids: Vec::new(),
            child_object_ids: vec![object_id],
            transform: translation(matrix.position),
        };
        root_ids.push(state.retain_hierarchy_node(node)?);
        matrices.push(QbExtMatrix {
            name: matrix.name.clone(),
            position: matrix.position,
            visibility,
        });
    }
    state.set_root_hierarchy_node_ids(root_ids)?;

    let qb_ext = QbExt {
        version: file.version,
        bgra: matches!(file.color_format, QbColorFormat::Bgra),
        right_handed: matches!(file.z_axis_orientation, QbZAxisOrientation::RightHanded),
        compressed: file.compressed,
        visibility_mask_encoded: file.visibility_mask_encoded,
        matrices,
    };

    Ok((state, qb_ext))
}

/// Builds the one shared palette: a color value pool of one entry per distinct
/// color across every matrix's solid voxels, bound to `baseColor`, with
/// one material per color and a map from a color to its material. The
/// value pool is added to `state`. A file with no solid voxels gets a single
/// placeholder color so objects have a default material to sample.
fn build_palette(
    state: &mut VoxMain<()>,
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
    let value_pool_id = state.retain_value_pool(
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

/// The visibility bytes of an object's solid voxels, in live-voxel raster
/// order, read back from the matrix the object was built from.
fn visibility_of(object: &VoxObject, matrix: &QbMatrix) -> Vec<u8> {
    object
        .iter_live()
        .map(|voxel_id| {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            matrix
                .voxel(position.x, position.y, position.z)
                .map_or(0, |voxel| voxel.visibility)
        })
        .collect()
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
