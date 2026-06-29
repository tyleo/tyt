use crate::{
    Error, QubicleQbExt, QubicleQbExtWrapper, QubicleQbMatrix, Result, tighten, to_vox_value,
};
use branded_id::U32Id;
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbZAxisOrientation};
use std::collections::{HashMap, HashSet};
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3U32};
use voxcore::{
    BVoxPalette, BVoxPaletteCell, VoxHierarchyNode, VoxObject, VoxPalette, VoxState, VoxValue,
};

/// Loads a decoded Qubicle Binary [`QbFile`] into a [`VoxState`].
///
/// Each matrix becomes an object sharing one `rgb` palette, placed by a hierarchy
/// node at the matrix's scene position. The state with no native voxcore home,
/// such as the header flags, matrix names, positions, and the per-voxel
/// visibility bytes, rides in a `qubicle-qb` ext so the file can be written back
/// exactly.
///
/// Errors on a matrix grid that exceeds the dense limit, or if
/// [`VoxState::validate`](voxcore::VoxState::validate) rejects the result.
pub fn from_qb_file(file: &QbFile) -> Result<VoxState> {
    let mut state = VoxState::default();

    let (palette, cells) = build_palette(file);
    let palette_id = state.add_palette(palette);

    let mut matrices = Vec::with_capacity(file.matrices.len());
    let mut roots = Vec::with_capacity(file.matrices.len());
    for matrix in &file.matrices {
        // The matrix grid may carry empty margin; the runtime object is tight, with
        // the matrix grid kept as the object's edit grid (build volume). The
        // visibility bytes are read from the original grid before tightening.
        let object = build_object(matrix, palette_id, &cells)?;
        let visibility = visibility_of(&object, matrix);
        let (object, edit) = tighten(&object);
        let object_id = state.add_object(object);
        state.set_edit_object(object_id, edit);
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

/// Builds the one shared palette: an `rgb` cell per distinct color across every
/// matrix's solid voxels, plus a map from a color to its cell. A file with no
/// solid voxels gets a single placeholder cell so objects have a default sample
/// to reference.
fn build_palette(file: &QbFile) -> (VoxPalette, HashMap<[u8; 3], U32Id<BVoxPaletteCell>>) {
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

    let mut palette = VoxPalette::default();
    palette.add_attribute("rgb".to_owned());
    let mut cells = HashMap::with_capacity(order.len());
    for color in &order {
        let cell = palette
            .add_cell(vec![VoxValue::Text(hex(*color))])
            .expect("one value per palette attribute");
        cells.insert(*color, cell);
    }

    (palette, cells)
}

/// Builds an object from a matrix: a dense grid sized by the matrix, referencing
/// the shared palette, each solid voxel sampling its color cell. Errors on an
/// oversized grid.
fn build_object(
    matrix: &QbMatrix,
    palette: U32Id<BVoxPalette>,
    cells: &HashMap<[u8; 3], U32Id<BVoxPaletteCell>>,
) -> Result<VoxObject> {
    let [size_x, size_y, size_z] = matrix.size;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size_x, size_y, size_z))
        .ok_or_else(|| {
            Error::invalid(format!(
                "matrix grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    object.add_palette_ref(palette, U32Id::<BVoxPaletteCell>::from_u32(0));

    for z in 0..size_z {
        for y in 0..size_y {
            for x in 0..size_x {
                let Some(voxel) = matrix.voxel(x, y, z) else {
                    continue;
                };
                if voxel.is_empty() {
                    continue;
                }
                let cell = cells
                    .get(&[voxel.r, voxel.g, voxel.b])
                    .copied()
                    .expect("every solid color is in the palette");
                let id = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .expect("a coordinate inside the matrix is inside the grid");
                object
                    .retain_voxel(id, &[cell])
                    .expect("one sample for the one palette reference");
            }
        }
    }

    Ok(object)
}

/// The visibility bytes of an object's solid voxels, in live-voxel raster order,
/// read back from the matrix the object was built from.
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
    TyTransformF64::new(
        TyVector3F64::new(position[0] as f64, position[1] as f64, position[2] as f64),
        TyQuaternionF64::identity(),
        TyVector3F64::new(1.0, 1.0, 1.0),
    )
}

/// One `#RRGGBB` string for an `[r, g, b]` color.
fn hex(color: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2])
}

#[cfg(test)]
mod tests {
    use crate::{from_qb_bytes, from_qb_file, to_qb_bytes, to_qb_file};
    use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
    use voxcore::VoxState;

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
        let state = VoxState::default();
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
