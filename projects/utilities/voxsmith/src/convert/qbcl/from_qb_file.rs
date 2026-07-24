use crate::{
    BASE_COLOR_FACTOR, Error, QubicleQbExt, QubicleQbExtWrapper, QubicleQbMatrix, Result,
    to_vox_value,
};
use branded_id::U32Id;
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbZAxisOrientation};
use std::collections::{HashMap, HashSet};
use ty_math::{TySrgbU8, TyTransformF64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxMaterial, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
};

/// Loads a decoded Qubicle Binary [`QbFile`] into a [`VoxMain`].
///
/// Each matrix becomes an object sharing one `baseColorFactor` palette, placed
/// by a hierarchy node at the matrix's scene position. The state with no native
/// voxcore home, such as the header flags, matrix names, positions, and the
/// per-voxel visibility bytes, rides in a `qubicle-qb` ext so the file can be
/// written back exactly.
///
/// Errors on a matrix grid that exceeds the dense limit, or if
/// [`VoxMain::validate`](voxcore::VoxMain::validate) rejects the result.
pub fn from_qb_file(file: &QbFile) -> Result<VoxMain> {
    let mut state = VoxMain::default();

    let (palette, materials) = build_palette(&mut state, file);
    let palette_id = state.add_palette(palette);

    let mut matrices = Vec::with_capacity(file.matrices.len());
    let mut roots = Vec::with_capacity(file.matrices.len());
    for matrix in &file.matrices {
        // The matrix grid becomes the object's build volume directly; it may
        // carry empty margin around the live voxels. The visibility bytes are
        // read from that same grid.
        let object = build_object(matrix, palette_id, &materials)?;
        let visibility = visibility_of(&object, matrix);
        let object_id = state.add_object(object);
        let node = VoxHierarchyNode {
            name: matrix.name.clone(),
            child_nodes: Vec::new(),
            child_objects: vec![object_id],
            transform: translation(matrix.position),
        };
        roots.push(state.add_hierarchy_node(node));
        matrices.push(QubicleQbMatrix {
            name: matrix.name.clone(),
            position: matrix.position,
            visibility,
        });
    }
    state.set_root_hierarchy_nodes(roots);

    let ext = QubicleQbExtWrapper {
        qubicle_qb: QubicleQbExt {
            version: file.version,
            bgra: matches!(file.color_format, QbColorFormat::Bgra),
            right_handed: matches!(file.z_axis_orientation, QbZAxisOrientation::RightHanded),
            compressed: file.compressed,
            visibility_mask_encoded: file.visibility_mask_encoded,
            matrices,
        },
    };
    state.set_ext(Some(to_vox_value(&ext)?));

    state.validate()?;
    Ok(state)
}

/// Builds the one shared palette: a color pool of one entry per distinct color
/// across every matrix's solid voxels, bound to `baseColorFactor`, with one
/// material per color and a map from a color to its material. The pool is added
/// to `state`. A file with no solid voxels gets a single placeholder color so
/// objects have a default material to sample.
fn build_palette(
    state: &mut VoxMain,
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

    // A Qubicle voxel carries no alpha, so colors ride in a shared sRGB pool as
    // float components in `[0, 1]`; each material draws one value id into it.
    let pool = state.add_value_pool(VoxValuePool::Srgb {
        values: order.iter().map(|&color| color_floats(color)).collect(),
    });

    let mut palette = VoxPalette::default();
    palette.add_property(BASE_COLOR_FACTOR.to_owned(), pool);
    let mut materials = HashMap::with_capacity(order.len());
    for (index, color) in order.iter().enumerate() {
        let material = palette
            .add_material(vec![U32Id::from_u32(index as u32)])
            .expect("one value id for the one property");
        materials.insert(*color, material);
    }

    (palette, materials)
}

/// Builds an object from a matrix: a dense grid sized by the matrix,
/// referencing the shared palette on one layer, each solid voxel sampling its
/// color material. Errors on an oversized grid.
fn build_object(
    matrix: &QbMatrix,
    palette: U32Id<BVoxPalette>,
    materials: &HashMap<[u8; 3], U32Id<BVoxMaterial>>,
) -> Result<VoxObject> {
    let [size_x, size_y, size_z] = matrix.size;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size_x, size_y, size_z))
        .ok_or_else(|| {
            Error::invalid(format!(
                "matrix grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    object.add_layer(palette, U32Id::<BVoxMaterial>::from_u32(0));

    for z in 0..size_z {
        for y in 0..size_y {
            for x in 0..size_x {
                let Some(voxel) = matrix.voxel(x, y, z) else {
                    continue;
                };
                if voxel.is_empty() {
                    continue;
                }
                let material = materials
                    .get(&[voxel.r, voxel.g, voxel.b])
                    .copied()
                    .expect("every solid color is in the palette");
                let id = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .expect("a coordinate inside the matrix is inside the grid");
                object
                    .retain_voxel(id, &[material])
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
        .map(|id| {
            let position = object
                .voxel_position(id)
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

/// The float sRGB components in `[0, 1]` of an `[r, g, b]` byte color.
fn color_floats(color: [u8; 3]) -> [f64; 3] {
    TySrgbU8::from(color).into_format::<f64>().into()
}

#[cfg(test)]
mod tests {
    use crate::{from_qb_bytes, from_qb_file, to_qb_bytes, to_qb_file};
    use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
    use voxcore::VoxMain;

    /// A file with two matrices: a `[2, 1, 1]` grid with two solid voxels, one
    /// carrying a non-default visibility mask, and a single-voxel grid.
    fn sample_file() -> QbFile {
        QbFile {
            version: 257,
            color_format: QbColorFormat::Bgra,
            z_axis_orientation: QbZAxisOrientation::RightHanded,
            compressed: true,
            visibility_mask_encoded: true,
            matrices: vec![
                QbMatrix {
                    name: "m0".to_owned(),
                    size: [2, 1, 1],
                    position: [1, 2, 3],
                    voxels: vec![
                        QbVoxel::new(10, 20, 30),
                        QbVoxel {
                            r: 1,
                            g: 2,
                            b: 3,
                            visibility: 0x3f,
                        },
                    ],
                },
                QbMatrix {
                    name: "m1".to_owned(),
                    size: [1, 1, 1],
                    position: [-1, -1, -1],
                    voxels: vec![QbVoxel::new(40, 50, 60)],
                },
            ],
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_qb_file(&file).unwrap();
        assert_eq!(to_qb_file(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_through_qb_bytes() {
        let file = sample_file();
        let state = from_qb_file(&file).unwrap();
        let bytes = to_qb_bytes(&state).unwrap();
        let reloaded = from_qb_bytes(&bytes).unwrap();
        assert_eq!(to_qb_file(&reloaded).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbFile::default();
        let state = from_qb_file(&file).unwrap();
        assert_eq!(to_qb_file(&state).unwrap(), file);
    }

    #[test]
    fn errors_without_qubicle_qb_ext() {
        let state = VoxMain::default();
        assert!(to_qb_file(&state).is_err());
    }

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
